use std::{future::Future, sync::Arc};

use anyhow::{Context, Result};
use futures::stream::StreamExt;

use cf_chains::sol::SolAddress;
use cf_primitives::EpochIndex;
use sol_rpc::{calls::GetGenesisHash, traits::CallApi as SolanaApi};
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

	let what = tokio::spawn(run(
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
	_sol_client: impl SolanaApi + Send + Sync + 'static,
	_process_call: ProcessCall,
	state_chain_client: Arc<StateChainClient>,
	_state_chain_stream: StateChainStream,
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
	let deposit_addresses_updates =
		deposit_addresses::deposit_addresses_updates(state_chain_client.as_ref());
	let mut deposit_addresses_updates = std::pin::pin!(deposit_addresses_updates);

	while let Some(update) = deposit_addresses_updates.next().await {
		tracing::warn!("DEPOSIT_ADDRESS_UPDATE: {:#?}", update);
	}

	Ok(())
}
