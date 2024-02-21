use std::{
	collections::HashMap,
	future::Future,
	sync::{atomic::AtomicBool, Arc},
	time::Duration,
};

use anyhow::{Context, Result};
use futures::{stream::StreamExt, FutureExt, TryFutureExt, TryStreamExt};

use cf_chains::sol::SolAddress;
use cf_primitives::EpochIndex;
use sol_rpc::{calls::GetGenesisHash, traits::CallApi as SolanaApi};
use sol_watch::{
	address_transactions_stream::AddressSignatures, deduplicate_stream::DeduplicateStreamExt,
	ensure_balance_continuity::EnsureBalanceContinuityStreamExt,
	fetch_balance::FetchBalancesStreamExt,
};
use utilities::task_scope::Scope;

use crate::{
	db::PersistentKeyDB,
	state_chain_observer::client::{
		chain_api::ChainApi,
		extrinsic_api::signed::SignedExtrinsicApi,
		storage_api::StorageApi,
		stream_api::{StreamApi, FINALIZED},
	},
};

use super::common::epoch_source::EpochSourceBuilder;

mod deposit_addresses;
mod sol_source;

const SOLANA_SIGNATURES_FOR_TRANSACTION_PAGE_SIZE: usize = 100;
const SOLANA_SIGNATURES_FOR_TRANSACTION_POLL_INTERVAL: Duration = Duration::from_secs(5);

pub async fn start<StateChainClient, StateChainStream, ProcessCall, ProcessingFut>(
	scope: &Scope<'_, anyhow::Error>,
	sol_client: impl SolanaApi + Send + Sync + 'static,
	process_call: ProcessCall,
	state_chain_client: Arc<StateChainClient>,
	state_chain_stream: StateChainStream,
	epoch_source: EpochSourceBuilder<'_, '_, StateChainClient, (), ()>,
	db: Arc<PersistentKeyDB>,
) -> Result<()>
where
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
	StateChainStream: StreamApi<FINALIZED> + Clone,
	ProcessCall: Fn(state_chain_runtime::RuntimeCall, EpochIndex) -> ProcessingFut
		+ Send
		+ Sync
		+ Clone
		+ 'static,
	ProcessingFut: Future<Output = ()> + Send + 'static,
{
	let solana_genesis_hash = sol_client.call(GetGenesisHash::default()).await?;
	tracing::info!("Solana genesis hash: {}", solana_genesis_hash);

	let vault_address = state_chain_client
		.storage_value::<pallet_cf_environment::SolanaVaultAddress<state_chain_runtime::Runtime>>(
			state_chain_client.latest_finalized_block().hash,
		)
		.await
		.context("Failed to get Vault contract address from SC")?;

	tracing::info!("solana vault address: {}", vault_address);

	scope.spawn(run(
		sol_client,
		process_call,
		state_chain_client,
		state_chain_stream,
		db,
		vault_address,
	));

	Ok(())
}

async fn run<StateChainClient, StateChainStream, ProcessCall, ProcessingFut>(
	sol_client: impl SolanaApi + Send + Sync + 'static,
	_process_call: ProcessCall,
	state_chain_client: Arc<StateChainClient>,
	state_chain_stream: StateChainStream,
	_db: Arc<PersistentKeyDB>,

	vault_address: SolAddress,
) -> Result<()>
where
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
	StateChainStream: StreamApi<FINALIZED> + Clone,
	ProcessCall: Fn(state_chain_runtime::RuntimeCall, EpochIndex) -> ProcessingFut
		+ Send
		+ Sync
		+ Clone
		+ 'static,
	ProcessingFut: Future<Output = ()> + Send + 'static,
{
	std::mem::drop(state_chain_stream);

	utilities::task_scope::task_scope(move |scope| {
		async move {
			let sol_client = Arc::new(sol_client);

			let deposit_addresses_updates =
				deposit_addresses::deposit_addresses_updates(state_chain_client.as_ref());
			let mut deposit_addresses_updates = std::pin::pin!(deposit_addresses_updates);

			let mut deposit_processor_kill_switches = HashMap::new();

			while let Some(update) = deposit_addresses_updates.next().await {
				for (channel_id, address) in update.added {
					tracing::warn!("DEPOSIT_ADDRESS_UPDATE: ADD [{}] {}", channel_id, address);

					let kill_switch = Arc::new(AtomicBool::default());
					deposit_processor_kill_switches.insert(channel_id, Arc::clone(&kill_switch));

					let running =
						AddressSignatures::new(Arc::clone(&sol_client), address, kill_switch)
							.max_page_size(SOLANA_SIGNATURES_FOR_TRANSACTION_PAGE_SIZE)
							.poll_interval(SOLANA_SIGNATURES_FOR_TRANSACTION_POLL_INTERVAL)
							// // TODO: find a way to start from where we may have left
							// .after_transaction(last_known_transaction)
							// // TODO: keep slot in the channel-state
							// .starting_with_slot(starting_with_slot)
							.into_stream()
							.deduplicate(
								SOLANA_SIGNATURES_FOR_TRANSACTION_PAGE_SIZE * 2,
								|r| r.as_ref().ok().copied(),
								|tx_id, _| {
									tracing::debug!(
										"AddressSignatures has returned a duplicate tx-id: {}",
										tx_id
									);
								},
							)
							.fetch_balances(Arc::clone(&sol_client), address)
							.map_err(anyhow::Error::from)
							.ensure_balance_continuity(SOLANA_SIGNATURES_FOR_TRANSACTION_PAGE_SIZE)
							.try_for_each(move |balance| async move {
								if let Some(deposited_amount) = balance.deposited() {
									tracing::warn!(
										"  DEPOSIT[{}] +{}; addr: {}; tx: {}",
										channel_id,
										deposited_amount,
										address,
										balance.signature,
									);
								}
								Ok(())
							})
							.map_err(Into::into);

					scope.spawn(running);
				}
				for (channel_id, address) in update.removed {
					tracing::warn!("DEPOSIT_ADDRESS_UPDATE: REM [{}] {}", channel_id, address);
				}
			}

			Ok(())
		}
		.boxed()
	})
	.await
}
