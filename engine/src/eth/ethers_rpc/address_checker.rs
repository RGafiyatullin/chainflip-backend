use ethers::prelude::*;

use anyhow::{Ok, Result};

use super::EthersRpcClient;

abigen!(AddressChecker, "eth-contract-abis/perseverance-rc17/IAddressChecker.json");

#[async_trait::async_trait]
pub trait AddressCheckerRpcApi {
	async fn address_states(
		&self,
		block_hash: H256,
		contract_address: H160,
		addresses: Vec<H160>,
	) -> Result<Vec<AddressState>>;

	async fn balances(
		&self,
		block_hash: H256,
		contract_address: H160,
		addresses: Vec<H160>,
	) -> Result<Vec<U256>>;
}

#[async_trait::async_trait]
impl AddressCheckerRpcApi for EthersRpcClient {
	async fn address_states(
		&self,
		block_hash: H256,
		contract_address: H160,
		addresses: Vec<H160>,
	) -> Result<Vec<AddressState>> {
		Ok(AddressChecker::new(contract_address, self.signer.inner().clone())
			.address_states(addresses)
			.block(BlockId::Hash(block_hash))
			.call()
			.await?)
	}

	async fn balances(
		&self,
		block_hash: H256,
		contract_address: H160,
		addresses: Vec<H160>,
	) -> Result<Vec<U256>> {
		Ok(AddressChecker::new(contract_address, self.signer.inner().clone())
			.native_balances(addresses)
			.block(BlockId::Hash(block_hash))
			.call()
			.await?)
	}
}
