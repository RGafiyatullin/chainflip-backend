use std::collections::{BTreeMap, BTreeSet};

use crate::{
	common::{Signal, Signaller},
	state_chain_observer::client,
};
use cf_primitives::EpochIndex;
use futures::StreamExt;
use futures_util::FutureExt;
use sp_core::H256;
use sp_runtime::AccountId32;
use utilities::{
	task_scope::{Scope, OR_CANCEL},
	UnendingStream,
};

const STATE_CHAIN_CONNECTION: &str = "State Chain client connection failed"; // TODO Replace with infallible SCC requests

pub struct Epoch {
	index: cf_primitives::EpochIndex,
	// A block hash that allows us to query epoch data validly.
	valid_block_hash: H256,
	expired_signal: Signal<H256>,
	not_current_signal: Signal<H256>,
}

pub struct ActiveEpochs {
	known: Vec<Epoch>,
	incoming: tokio_stream::wrappers::UnboundedReceiverStream<Epoch>,
}

pub struct Client {
	request_sender: async_channel::Sender<tokio::sync::oneshot::Sender<ActiveEpochs>>,
}

impl Client {
	pub async fn new<
		'env,
		StateChainStream: client::StateChainStreamApi,
		StateChainClient: client::storage_api::StorageApi + Send + Sync + 'static,
	>(
		scope: &Scope<'env, anyhow::Error>,
		mut state_chain_stream: StateChainStream,
		state_chain_client: StateChainClient,
	) -> Client {
		struct SignallerAndSignal {
			signaller: Signaller<H256>,
			signal: Signal<H256>,
		}
		impl SignallerAndSignal {
			fn new() -> Self {
				let (signaller, signal) = Signal::new();
				SignallerAndSignal { signaller, signal }
			}
		}

		struct CurrentEpoch {
			index: EpochIndex,
			not_current: SignallerAndSignal,
			expired: SignallerAndSignal,
		}
		impl CurrentEpoch {
			fn new(index: EpochIndex) -> Self {
				CurrentEpoch {
					index,
					expired: SignallerAndSignal::new(),
					not_current: SignallerAndSignal::new(),
				}
			}
		}

		let (request_sender, mut request_receiver) =
			async_channel::unbounded::<tokio::sync::oneshot::Sender<ActiveEpochs>>();

		let initial_block_hash = state_chain_stream.cache().block_hash;

		// TODO: Handle the fact that this contains both epochs.
		let mut epoch_expiries = BTreeMap::from_iter(
			state_chain_client
				.storage_map::<pallet_cf_validator::EpochExpiries<state_chain_runtime::Runtime>>(
					initial_block_hash,
				)
				.await
				.expect(STATE_CHAIN_CONNECTION)
				.into_iter()
				.map(|(_, index)| (index, SignallerAndSignal::new())),
		);

		let mut current_epoch = CurrentEpoch::new(
			state_chain_client
				.storage_value::<pallet_cf_validator::CurrentEpoch<state_chain_runtime::Runtime>>(
					initial_block_hash,
				)
				.await
				.expect(STATE_CHAIN_CONNECTION),
		);

		assert!(epoch_expiries.contains_key(&current_epoch.index));

		let mut epoch_senders = Vec::<tokio::sync::mpsc::UnboundedSender<Epoch>>::new();

		scope.spawn(async move {
			utilities::loop_select! {
				if request_receiver.is_closed() && epoch_senders.is_empty() => let _ = futures::future::ready(()) => {
					break Ok(())
				},
				if !epoch_senders.is_empty() /* select_all panics if iter empty */ => let _ = futures::future::select_all(epoch_senders.iter().map(|epoch_sender| Box::pin(epoch_sender.closed()))).map(|_| ()) => {
					epoch_senders.retain(|epoch_sender| !epoch_sender.is_closed());
				},
				let response_sender = request_receiver.next_or_pending() => {
					let (epoch_sender, epoch_receiver) = tokio::sync::mpsc::unbounded_channel();
					epoch_senders.push(epoch_sender);
					let _result = response_sender.send(ActiveEpochs {
						known: (epoch_expiries.iter().map(|(index, expired)| {
							Epoch {
								index: *index,
								valid_block_hash: initial_block_hash,
								expired_signal: expired.signal.clone(),
								not_current_signal: Signal::signalled(initial_block_hash),
							}
						}).chain(std::iter::once({
							Epoch {
								index: current_epoch.index,
								valid_block_hash: initial_block_hash,
								expired_signal: current_epoch.expired.signal.clone(),
								not_current_signal: current_epoch.not_current.signal.clone(),
							}
						}))).collect(),
						incoming: tokio_stream::wrappers::UnboundedReceiverStream::new(epoch_receiver),
					});
				},
				if let Some((block_hash, _block_header)) = state_chain_stream.next() => {
					let new_current_epoch = state_chain_client
						.storage_value::<pallet_cf_validator::CurrentEpoch<
							state_chain_runtime::Runtime,
						>>(block_hash)
						.await
						.expect(STATE_CHAIN_CONNECTION);

					if new_current_epoch != current_epoch.index {
						assert!(epoch_expiries.contains_key(&new_current_epoch));

						current_epoch.not_current.signaller.signal(block_hash);
						epoch_expiries.insert(current_epoch.index, current_epoch.expired);

						current_epoch = CurrentEpoch::new(new_current_epoch);

						for epoch_sender in &epoch_senders {
							let _result = epoch_sender.send(Epoch {
								index: current_epoch.index,
								valid_block_hash: block_hash,
								expired_signal: current_epoch.expired.signal.clone(),
								not_current_signal: current_epoch.not_current.signal.clone(),
							});
						}
					}

					let new_epoch_expiries = BTreeMap::from_iter(
						state_chain_client.storage_map::<pallet_cf_validator::EpochExpiries<
							state_chain_runtime::Runtime,
						>>(block_hash).await.expect(STATE_CHAIN_CONNECTION).into_iter().map(|(_, index)| {
							(index, if let Some(historical_active_epoch) = epoch_expiries.remove(&index) {
								historical_active_epoch
							} else {
								let expired = SignallerAndSignal::new();

								for epoch_sender in &epoch_senders {
									let _result = epoch_sender.send(Epoch {
										index,
										valid_block_hash: block_hash,
										expired_signal: expired.signal.clone(),
										not_current_signal: Signal::signalled(block_hash),
									});
								}

								expired
							})
						})
					);

					for (_, expired) in epoch_expiries {
						expired.signaller.signal(block_hash);
					}

					epoch_expiries = new_epoch_expiries;

					assert!(epoch_expiries.contains_key(&current_epoch.index));
				} else break Ok(()),
			}
		});

		Self { request_sender }
	}

	pub async fn active_epochs(&self) -> ActiveEpochs {
		let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
		drop(self.request_sender.send(response_sender));
		response_receiver.await.expect(OR_CANCEL)
	}
}

impl ActiveEpochs {
	pub async fn filter_by_participation<
		StateChainClient: client::storage_api::StorageApi + Clone + Send + Sync + 'static,
	>(
		mut self,
		scope: &Scope<'_, StateChainClient>,
		account_id: AccountId32,
		state_chain_client: StateChainClient,
	) -> ActiveEpochs {
		let state_chain_client = state_chain_client.clone();
		let known_and_participating = futures::stream::iter(self.known)
			.filter_map(|epoch| {
				let state_chain_client = state_chain_client.clone();
				let account_id = account_id.clone();
				async move {
					let historical_active_epochs = BTreeSet::from_iter(
						state_chain_client
							.storage_map_entry::<pallet_cf_validator::HistoricalActiveEpochs<
								state_chain_runtime::Runtime,
							>>(epoch.valid_block_hash, &account_id)
							.await
							.unwrap(),
					);
					if historical_active_epochs.contains(&epoch.index) {
						Some(epoch)
					} else {
						None
					}
				}
			})
			.collect::<Vec<_>>()
			.await;

		let (epoch_sender, epoch_receiver) = tokio::sync::mpsc::unbounded_channel();

		scope.spawn(async move {
			utilities::loop_select! {
				if epoch_sender.is_closed() => let _ = futures::future::ready(()) => {
					break Ok(())
				},
				let epoch = self.incoming.next_or_pending() => {
					let historical_active_epochs = BTreeSet::from_iter(
						state_chain_client
							.storage_map_entry::<pallet_cf_validator::HistoricalActiveEpochs<state_chain_runtime::Runtime>>(
								epoch.valid_block_hash,
								&account_id,
							)
							.await
							.unwrap(),
					);

					if historical_active_epochs.contains(&epoch.index) {
						let _result = epoch_sender.send(epoch);
					}
				},
			}
		});

		ActiveEpochs {
			known: known_and_participating,
			incoming: tokio_stream::wrappers::UnboundedReceiverStream::new(epoch_receiver),
		}
	}
}

#[async_trait::async_trait]
pub trait GetVaultData {
	type VaultData: Send + Sync + 'static;

	async fn get_vault_data(&self, epoch: &Epoch) -> Self::VaultData;
}

pub struct Vault<C: cf_chains::Chain> {
	active_from_block: C::ChainBlockNumber,
	start_data: C::EpochStartData,
	// end_block: Option<C::ChainBlockNumber>,
}

pub struct ActiveVaults<C: cf_chains::Chain> {
	known: Vec<Vault<C>>,
	incoming: tokio_stream::wrappers::UnboundedReceiverStream<Vault<C>>,
}

async fn active_epochs_vault_data<C, I, StateChainClient, VaultDataGetter>(
	mut active_epochs: ActiveEpochs,
	scope: &Scope<'_, StateChainClient>,
	state_chain_client: StateChainClient,
	chain_client: VaultDataGetter,
) -> ActiveVaults<C>
where
	C: cf_chains::ChainAbi,
	I: 'static + Send + Sync,
	StateChainClient: client::storage_api::StorageApi + Clone + Send + Sync + 'static,
	VaultDataGetter: GetVaultData<VaultData = C::EpochStartData> + Clone + Send + Sync + 'static,
	state_chain_runtime::Runtime: pallet_cf_vaults::Config<I, Chain = C>,
{
	// We have a vault for each epoch
	let known_vaults = futures::stream::iter(active_epochs.known)
		.filter_map(|epoch| {
			let state_chain_client = state_chain_client.clone();
			let chain_client = chain_client.clone();
			async move {
				if let Some(vault) = state_chain_client
					.storage_map_entry::<pallet_cf_vaults::Vaults<state_chain_runtime::Runtime, I>>(
						epoch.valid_block_hash,
						&epoch.index,
					)
					.await
					.unwrap()
				{
					Some(Vault {
						active_from_block: vault.active_from_block,
						start_data: chain_client.get_vault_data(&epoch).await,
					})
				} else {
					None
				}
			}
		})
		.collect::<Vec<_>>()
		.await;

	let (vault_sender, vault_receiver) = tokio::sync::mpsc::unbounded_channel();

	scope.spawn(async move {
		utilities::loop_select! {
			if vault_sender.is_closed() => let _ = futures::future::ready(()) => {
				break Ok(())
			},
			let epoch = active_epochs.incoming.next_or_pending() => {
				if let Some(vault) = state_chain_client
					.storage_map_entry::<pallet_cf_vaults::Vaults<state_chain_runtime::Runtime, I>>(
						epoch.valid_block_hash,
						&epoch.index,
					)
					.await
					.unwrap() {
					let _result = vault_sender.send(Vault { active_from_block: vault.active_from_block, start_data: chain_client.get_vault_data(&epoch).await });
				}
			},
		}
	});

	ActiveVaults {
		known: known_vaults,
		incoming: tokio_stream::wrappers::UnboundedReceiverStream::new(vault_receiver),
	}
}
