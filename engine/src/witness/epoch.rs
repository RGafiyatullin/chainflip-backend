use std::collections::BTreeMap;

use crate::{
	common::{Signal, Signaller},
	state_chain_observer::client,
};
use cf_primitives::EpochIndex;
use futures::StreamExt;
use futures_core::Stream;
use futures_util::FutureExt;
use sp_core::H256;
use utilities::{
	task_scope::{Scope, OR_CANCEL},
	UnendingStream,
};

const STATE_CHAIN_CONNECTION: &str = "State Chain client connection failed"; // TODO Replace with infallible SCC requests

pub struct Epoch<Data> {
	index: cf_primitives::EpochIndex,
	expired_signal: Signal<H256>,
	not_current_signal: Signal<H256>,
	data: Data,
}

pub struct Client {
	request_sender: async_channel::Sender<
		tokio::sync::oneshot::Sender<(
			Vec<Epoch<()>>,
			tokio_stream::wrappers::UnboundedReceiverStream<Epoch<()>>,
		)>,
	>,
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
	) -> Self {
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

		let (request_sender, mut request_receiver) = async_channel::unbounded::<
			tokio::sync::oneshot::Sender<(
				Vec<Epoch<()>>,
				tokio_stream::wrappers::UnboundedReceiverStream<Epoch<()>>,
			)>,
		>();

		let initial_block_hash = state_chain_stream.cache().block_hash;

		let mut historical_active_epochs = BTreeMap::from_iter(
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

		assert!(historical_active_epochs.contains_key(&current_epoch.index));

		let mut epoch_senders = Vec::<tokio::sync::mpsc::UnboundedSender<Epoch<()>>>::new();

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
					let _result = response_sender.send((
						(historical_active_epochs.iter().map(|(index, expired)| {
							Epoch {
								index: *index,
								expired_signal: expired.signal.clone(),
								not_current_signal: Signal::signalled(initial_block_hash),
								data: (),
							}
						}).chain(std::iter::once({
							Epoch {
								index: current_epoch.index,
								expired_signal: current_epoch.expired.signal.clone(),
								not_current_signal: current_epoch.not_current.signal.clone(),
								data: (),
							}
						}))).collect(),
						tokio_stream::wrappers::UnboundedReceiverStream::new(epoch_receiver)
					));
				},
				if let Some((block_hash, _block_header)) = state_chain_stream.next() => {
					let new_current_epoch = state_chain_client
						.storage_value::<pallet_cf_validator::CurrentEpoch<
							state_chain_runtime::Runtime,
						>>(block_hash)
						.await
						.expect(STATE_CHAIN_CONNECTION);

					if new_current_epoch != current_epoch.index {
						assert!(historical_active_epochs.contains_key(&new_current_epoch));

						current_epoch.not_current.signaller.signal(block_hash);
						historical_active_epochs.insert(current_epoch.index, current_epoch.expired);

						current_epoch = CurrentEpoch::new(new_current_epoch);

						for epoch_sender in &epoch_senders {
							let _result = epoch_sender.send(Epoch {
								index: current_epoch.index,
								expired_signal: current_epoch.expired.signal.clone(),
								not_current_signal: current_epoch.not_current.signal.clone(),
								data: (),
							});
						}
					}

					let new_historical_active_epochs = BTreeMap::from_iter(
						state_chain_client.storage_map::<pallet_cf_validator::EpochExpiries<
							state_chain_runtime::Runtime,
						>>(block_hash).await.expect(STATE_CHAIN_CONNECTION).into_iter().map(|(_, index)| {
							(index, if let Some(historical_active_epoch) = historical_active_epochs.remove(&index) {
								historical_active_epoch
							} else {
								let expired = SignallerAndSignal::new();

								for epoch_sender in &epoch_senders {
									let _result = epoch_sender.send(Epoch {
										index,
										expired_signal: expired.signal.clone(),
										not_current_signal: Signal::signalled(block_hash),
										data: (),
									});
								}

								expired
							})
						})
					);

					for (_, expired) in historical_active_epochs {
						expired.signaller.signal(block_hash);
					}

					historical_active_epochs = new_historical_active_epochs;

					assert!(historical_active_epochs.contains_key(&current_epoch.index));
				} else break Ok(()),
			}
		});

		Self { request_sender }
	}

	// Structure instead of tuple
	// apply filter to get participants

	// trait to map to get epoch start data / vault, for each chain.
	pub async fn active_epochs(&self) -> (Vec<Epoch<()>>, impl Stream<Item = Epoch<()>>) {
		let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
		drop(self.request_sender.send(response_sender));
		response_receiver.await.expect(OR_CANCEL)
	}
}
