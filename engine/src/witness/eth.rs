mod contract_common;
mod erc20_deposits;
mod eth_chain_tracking;
mod eth_source;
mod ethereum_deposits;
mod key_manager;
mod state_chain_gateway;
pub mod vault;

use std::{collections::HashMap, sync::Arc};

use cf_primitives::{chains::assets::eth, EpochIndex};
use futures_core::Future;
use sp_core::H160;
use utilities::task_scope::Scope;

use crate::{
	db::PersistentKeyDB,
	eth::retry_rpc::EthersRetryRpcClient,
	state_chain_observer::client::{
		chain_api::ChainApi, extrinsic_api::signed::SignedExtrinsicApi, storage_api::StorageApi,
		StateChainStreamApi,
	},
	witness::eth::erc20_deposits::{flip::FlipEvents, usdc::UsdcEvents},
};

use super::common::{
	chain_source::extension::ChainSourceExt, epoch_source::EpochSourceBuilder,
	STATE_CHAIN_CONNECTION,
};
use eth_source::EthSource;

use anyhow::{Context, Result};

const SAFETY_MARGIN: usize = 7;

pub async fn start<StateChainClient, StateChainStream, ProcessCall, ProcessingFut>(
	scope: &Scope<'_, anyhow::Error>,
	eth_client: EthersRetryRpcClient,
	process_call: ProcessCall,
	state_chain_client: Arc<StateChainClient>,
	state_chain_stream: StateChainStream,
	epoch_source: EpochSourceBuilder<'_, '_, StateChainClient, (), ()>,
	db: Arc<PersistentKeyDB>,
) -> Result<()>
where
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
	StateChainStream: StateChainStreamApi + Clone,
	ProcessCall: Fn(state_chain_runtime::RuntimeCall, EpochIndex) -> ProcessingFut
		+ Send
		+ Sync
		+ Clone
		+ 'static,
	ProcessingFut: Future<Output = ()> + Send + 'static,
{
	let state_chain_gateway_address = state_chain_client
        .storage_value::<pallet_cf_environment::EthereumStateChainGatewayAddress<state_chain_runtime::Runtime>>(
            state_chain_client.latest_finalized_hash(),
        )
        .await
        .context("Failed to get StateChainGateway address from SC")?;

	let key_manager_address = state_chain_client
		.storage_value::<pallet_cf_environment::EthereumKeyManagerAddress<state_chain_runtime::Runtime>>(
			state_chain_client.latest_finalized_hash(),
		)
		.await
		.context("Failed to get KeyManager address from SC")?;

	let vault_address = state_chain_client
		.storage_value::<pallet_cf_environment::EthereumVaultAddress<state_chain_runtime::Runtime>>(
			state_chain_client.latest_finalized_hash(),
		)
		.await
		.context("Failed to get Vault contract address from SC")?;

	let address_checker_address = state_chain_client
		.storage_value::<pallet_cf_environment::EthereumAddressCheckerAddress<state_chain_runtime::Runtime>>(
			state_chain_client.latest_finalized_hash(),
		)
		.await
		.expect(STATE_CHAIN_CONNECTION);

	let supported_erc20_tokens: HashMap<cf_primitives::chains::assets::eth::Asset, H160> =
		state_chain_client
			.storage_map::<pallet_cf_environment::EthereumSupportedAssets<state_chain_runtime::Runtime>, _>(
				state_chain_client.latest_finalized_hash(),
			)
			.await
			.context("Failed to fetch Ethereum supported assets")?;

	let usdc_contract_address =
		*supported_erc20_tokens.get(&eth::Asset::Usdc).context("USDC not supported")?;

	let flip_contract_address =
		*supported_erc20_tokens.get(&eth::Asset::Flip).context("FLIP not supported")?;

	let supported_erc20_tokens: HashMap<H160, cf_primitives::Asset> = supported_erc20_tokens
		.into_iter()
		.map(|(asset, address)| (address, asset.into()))
		.collect();

	let eth_source = EthSource::new(eth_client.clone()).shared(scope);

	eth_source
		.clone()
		.shared(scope)
		.chunk_by_time(epoch_source.clone())
		.chain_tracking(state_chain_client.clone(), eth_client.clone())
		.logging("chain tracking")
		.spawn(scope);

	let eth_safe_vault_source = eth_source
		.strictly_monotonic()
		.lag_safety(SAFETY_MARGIN)
		.logging("safe block produced")
		.shared(scope)
		.chunk_by_vault(epoch_source.vaults().await);

	eth_safe_vault_source
		.clone()
		.key_manager_witnessing(process_call.clone(), eth_client.clone(), key_manager_address)
		.continuous("KeyManager".to_string(), db.clone())
		.logging("KeyManager")
		.spawn(scope);

	eth_safe_vault_source
		.clone()
		.state_chain_gateway_witnessing(
			process_call.clone(),
			eth_client.clone(),
			state_chain_gateway_address,
		)
		.continuous("StateChainGateway".to_string(), db.clone())
		.logging("StateChainGateway")
		.spawn(scope);

	eth_safe_vault_source
		.clone()
		.deposit_addresses(scope, state_chain_stream.clone(), state_chain_client.clone())
		.await
		.erc20_deposits::<_, _, _, UsdcEvents>(
			process_call.clone(),
			eth_client.clone(),
			cf_primitives::chains::assets::eth::Asset::Usdc,
			usdc_contract_address,
		)
		.await?
		.continuous("USDCDeposits".to_string(), db.clone())
		.logging("USDCDeposits")
		.spawn(scope);

	eth_safe_vault_source
		.clone()
		.deposit_addresses(scope, state_chain_stream.clone(), state_chain_client.clone())
		.await
		.erc20_deposits::<_, _, _, FlipEvents>(
			process_call.clone(),
			eth_client.clone(),
			cf_primitives::chains::assets::eth::Asset::Flip,
			flip_contract_address,
		)
		.await?
		.continuous("FlipDeposits".to_string(), db.clone())
		.logging("FlipDeposits")
		.spawn(scope);

	eth_safe_vault_source
		.clone()
		.deposit_addresses(scope, state_chain_stream.clone(), state_chain_client.clone())
		.await
		.ethereum_deposits(
			process_call.clone(),
			eth_client.clone(),
			eth::Asset::Eth,
			address_checker_address,
			vault_address,
		)
		.await
		.continuous("EthereumDeposits".to_string(), db.clone())
		.logging("EthereumDeposits")
		.spawn(scope);

	eth_safe_vault_source
		.vault_witnessing(
			process_call,
			eth_client.clone(),
			vault_address,
			cf_primitives::Asset::Eth,
			cf_primitives::ForeignChain::Ethereum,
			supported_erc20_tokens,
		)
		.continuous("Vault".to_string(), db)
		.logging("Vault")
		.spawn(scope);

	Ok(())
}
