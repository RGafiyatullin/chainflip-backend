use std::{
	collections::HashMap,
	future::Future,
	sync::{atomic::AtomicBool, Arc},
	time::Duration,
};

use anyhow::{Context, Result};
use futures::{stream::StreamExt, FutureExt, TryStreamExt};

use cf_chains::{sol::SolAddress, ChainState};
use cf_primitives::{ChannelId, EpochIndex};
use sol_prim::SlotNumber;
use sol_rpc::{calls::GetGenesisHash, traits::CallApi as SolanaApi};
use sol_watch::{
	address_transactions_stream::AddressSignatures, deduplicate_stream::DeduplicateStreamExt,
	ensure_balance_continuity::EnsureBalanceContinuityStreamExt,
	fetch_balance::FetchBalancesStreamExt,
};
use state_chain_runtime::SolanaInstance;
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
mod tracked_data;

const SOLANA_SIGNATURES_FOR_TRANSACTION_PAGE_SIZE: usize = 100;
const SOLANA_SIGNATURES_FOR_TRANSACTION_POLL_INTERVAL: Duration = Duration::from_secs(5);
const SOLANA_CHAIN_TRACKER_SLEEP_INTERVAL: Duration = Duration::from_secs(5);

pub async fn start<SolanaClient, StateChainClient, StateChainStream, ProcessCall, ProcessingFut>(
	scope: &Scope<'_, anyhow::Error>,
	sol_client: SolanaClient,
	process_call: ProcessCall,
	state_chain_client: Arc<StateChainClient>,
	_state_chain_stream: StateChainStream,
	_epoch_source: EpochSourceBuilder<'_, '_, StateChainClient, (), ()>,
	_db: Arc<PersistentKeyDB>,
) -> Result<()>
where
	SolanaClient: SolanaApi + Send + Sync + 'static,
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
	StateChainStream: StreamApi<FINALIZED> + Clone,
	ProcessCall: Fn(state_chain_runtime::RuntimeCall, EpochIndex) -> ProcessingFut
		+ Send
		+ Sync
		+ Clone
		+ 'static,
	ProcessingFut: Future<Output = ()> + Send + 'static,
{
	let sol_client = Arc::new(sol_client);

	let solana_genesis_hash = sol_client.call(GetGenesisHash::default()).await?;
	tracing::info!("Solana genesis hash: {}", solana_genesis_hash);

	let vault_address = state_chain_client
		.storage_value::<pallet_cf_environment::SolanaVaultAddress<state_chain_runtime::Runtime>>(
			state_chain_client.latest_finalized_block().hash,
		)
		.await
		.context("Failed to get Vault contract address from SC")?;

	tracing::info!("solana vault address: {}", vault_address);

	scope.spawn(track_chain_state(
		Arc::clone(&sol_client),
		process_call.clone(),
		state_chain_client.clone(),
	));

	scope.spawn(track_deposit_addresses(sol_client, process_call, state_chain_client));

	Ok(())
}

async fn track_chain_state<SolanaClient, StateChainClient, ProcessCall, ProcessingFut>(
	sol_client: Arc<SolanaClient>,
	process_call: ProcessCall,
	state_chain_client: Arc<StateChainClient>,
) -> Result<()>
where
	SolanaClient: SolanaApi + Send + Sync + 'static,
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
	ProcessCall: Fn(state_chain_runtime::RuntimeCall, EpochIndex) -> ProcessingFut
		+ Send
		+ Sync
		+ Clone
		+ 'static,
	ProcessingFut: Future<Output = ()> + Send + 'static,
{
	let mut ticks = tokio::time::interval(SOLANA_CHAIN_TRACKER_SLEEP_INTERVAL);
	let mut last_reported_height = None;

	loop {
		ticks.tick().await;

		let chain_state = tracked_data::collect_tracked_data(&sol_client).await?;

		if last_reported_height.replace(chain_state.block_height) != Some(chain_state.block_height)
		{
			tracing::info!("updating chain state with {:?}", chain_state);

			let call = pallet_cf_chain_tracking::Call::<
				state_chain_runtime::Runtime,
				SolanaInstance,
			>::update_chain_state {
				new_chain_state: chain_state,
			};

			process_call(call.into(), 1).await;
		}
	}
}

async fn track_deposit_addresses<SolanaClient, StateChainClient, ProcessCall, ProcessingFut>(
	sol_client: Arc<SolanaClient>,
	_process_call: ProcessCall,
	state_chain_client: Arc<StateChainClient>,
) -> Result<()>
where
	SolanaClient: SolanaApi + Send + Sync + 'static,
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
	ProcessCall: Fn(state_chain_runtime::RuntimeCall, EpochIndex) -> ProcessingFut
		+ Send
		+ Sync
		+ Clone
		+ 'static,
	ProcessingFut: Future<Output = ()> + Send + 'static,
{
	// std::mem::drop(state_chain_stream);

	utilities::task_scope::task_scope(move |scope| {
		async move {
			let deposit_addresses_updates =
				deposit_addresses::deposit_addresses_updates(state_chain_client.as_ref());
			let mut deposit_addresses_updates = std::pin::pin!(deposit_addresses_updates);

			let mut deposit_processor_kill_switches = HashMap::new();

			while let Some(update) = deposit_addresses_updates.next().await.transpose()? {
				for (channel_id, address, channel_state) in update.added {
					tracing::info!(
						"starting up a deposit-address tracking for #{}: {} [{:?}]",
						channel_id,
						address,
						channel_state
					);

					let kill_switch = Arc::new(AtomicBool::default());
					deposit_processor_kill_switches.insert(channel_id, Arc::clone(&kill_switch));

					let running = track_single_deposit_address(
						Arc::clone(&sol_client),
						channel_id,
						address,
						channel_state.active_since_slot_number,
						kill_switch,
					);
					scope.spawn(running);
				}
				for (channel_id, address) in update.removed {
					tracing::info!(
						"shutting down a deposit-address tracking for #{}: {}",
						channel_id,
						address
					);

					if let Some(kill_switch) = deposit_processor_kill_switches.remove(&channel_id) {
						kill_switch.store(false, std::sync::atomic::Ordering::Relaxed);
					} else {
						tracing::warn!("Could not find a kill-switch for channel #{}", channel_id);
					}
				}
			}

			Ok(())
		}
		.boxed()
	})
	.await
}

async fn track_single_deposit_address<SolanaClient>(
	sol_client: Arc<SolanaClient>,
	channel_id: ChannelId,
	address: SolAddress,
	active_since_slot_number: SlotNumber,
	kill_switch: Arc<AtomicBool>,
) -> Result<()>
where
	SolanaClient: SolanaApi + Send + Sync + 'static,
{
	AddressSignatures::new(Arc::clone(&sol_client), address, kill_switch)
		.max_page_size(SOLANA_SIGNATURES_FOR_TRANSACTION_PAGE_SIZE)
		.poll_interval(SOLANA_SIGNATURES_FOR_TRANSACTION_POLL_INTERVAL)
		// // TODO: find a way to start from where we may have left
		// .after_transaction(last_known_transaction)
		.starting_with_slot(active_since_slot_number)
		.into_stream()
		.deduplicate(
			SOLANA_SIGNATURES_FOR_TRANSACTION_PAGE_SIZE * 2,
			|r| r.as_ref().ok().copied(),
			|tx_id, _| {
				tracing::debug!("AddressSignatures has returned a duplicate tx-id: {}", tx_id);
			},
		)
		.fetch_balances(Arc::clone(&sol_client), address)
		.map_err(anyhow::Error::from)
		.ensure_balance_continuity(SOLANA_SIGNATURES_FOR_TRANSACTION_PAGE_SIZE)
		.try_for_each(move |balance| async move {
			if let Some(deposited_amount) = balance.deposited() {
				tracing::info!(
					"  deposit-address #{}: +{} lamports; [addr: {}; tx: {}]",
					channel_id,
					deposited_amount,
					address,
					balance.signature,
				);
			}
			Ok(())
		})
		.await?;
	Ok(())
}
