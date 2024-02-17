use std::{future::Future, sync::Arc, time::Duration};

use anyhow::{Context, Result};

use cf_primitives::EpochIndex;
use sol_rpc::traits::CallApi as SolanaApi;
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

mod sol_source;

pub async fn start<StateChainClient, StateChainStream, ProcessCall, ProcessingFut>(
	_scope: &Scope<'_, anyhow::Error>,
	_sol_client: impl SolanaApi,
	_process_call: ProcessCall,
	state_chain_client: Arc<StateChainClient>,
	_state_chain_stream: StateChainStream,
	_epoch_source: EpochSourceBuilder<'_, '_, StateChainClient, (), ()>,
	_db: Arc<PersistentKeyDB>,
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
	let vault_address = state_chain_client
		.storage_value::<pallet_cf_environment::SolanaVaultAddress<state_chain_runtime::Runtime>>(
			state_chain_client.latest_finalized_block().hash,
		)
		.await
		.context("Failed to get Vault contract address from SC")?;

	tracing::info!("solana vault address: {}", vault_address);

	tokio::spawn(poll_deposit_addresses(state_chain_client.clone()));

	// let latest_blockhash = sol_client.get_latest_blockhash(Default::default()).await;

	Ok(())
}

const SC_BLOCK_TIME: Duration = Duration::from_secs(6);

async fn poll_deposit_addresses<StateChainClient>(
	state_chain_client: Arc<StateChainClient>,
) -> Result<()>
where
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
{
	let mut interval = tokio::time::interval(SC_BLOCK_TIME);

	loop {
		let _at = interval.tick().await;

		let deposit_addresses = state_chain_client
			.storage_map_values::<pallet_cf_ingress_egress::DepositChannelLookup<
				state_chain_runtime::Runtime,
				SolanaInstance,
			>>(state_chain_client.latest_finalized_block().hash)
			.await?;

		tracing::info!("deposit addresses: {:#?}", deposit_addresses);
	}
}
