use std::sync::Arc;

use async_trait::async_trait;
use cf_chains::eth::Ethereum;
use sp_core::H160;
use state_chain_runtime::EthereumInstance;
use tokio::sync::Mutex;

use crate::{
	eth::{core_h160, core_h256},
	state_chain_observer::client::extrinsic_api::signed::SignedExtrinsicApi,
	witnesser::{EpochStart, ItemMonitor},
};

use super::{eth_block_witnessing::BlockProcessor, rpc::EthDualRpcClient, EthNumberBloom};

pub struct DepositWitnesser<StateChainClient> {
	rpc: EthDualRpcClient,
	state_chain_client: Arc<StateChainClient>,
	address_monitor: Arc<Mutex<ItemMonitor<H160, H160, ()>>>,
}

impl<StateChainClient> DepositWitnesser<StateChainClient>
where
	StateChainClient: SignedExtrinsicApi + Send + Sync,
{
	pub fn new(
		state_chain_client: Arc<StateChainClient>,
		rpc: EthDualRpcClient,
		address_monitor: Arc<Mutex<ItemMonitor<H160, H160, ()>>>,
	) -> Self {
		Self { rpc, state_chain_client, address_monitor }
	}
}

#[async_trait]
impl<StateChainClient> BlockProcessor for DepositWitnesser<StateChainClient>
where
	StateChainClient: SignedExtrinsicApi + Send + Sync,
{
	async fn process_block(
		&mut self,
		epoch: &EpochStart<Ethereum>,
		block: &EthNumberBloom,
	) -> anyhow::Result<()> {
		use crate::eth::rpc::EthRpcApi;
		use cf_primitives::chains::assets::eth;
		use pallet_cf_ingress_egress::DepositWitness;

		let mut address_monitor =
			self.address_monitor.try_lock().expect("should have exclusive ownership");

		// Before we process the transactions, check if
		// we have any new addresses to monitor
		address_monitor.sync_items();

		let deposit_witnesses = self
			.rpc
			.successful_transactions(block.block_number)
			.await?
			.iter()
			.filter_map(|tx| {
				let to_addr = core_h160(tx.to?);
				if address_monitor.contains(&to_addr) {
					Some((tx, to_addr))
				} else {
					None
				}
			})
			.map(|(tx, to_addr)| DepositWitness {
				deposit_address: to_addr,
				asset: eth::Asset::Eth,
				amount: tx
					.value
					.try_into()
					.expect("Ingress witness transfer value should fit u128"),
				tx_id: core_h256(tx.hash),
			})
			.collect::<Vec<DepositWitness<Ethereum>>>();

		if !deposit_witnesses.is_empty() {
			self.state_chain_client
				.submit_signed_extrinsic(pallet_cf_witnesser::Call::witness_at_epoch {
					call: Box::new(
						pallet_cf_ingress_egress::Call::<_, EthereumInstance>::process_deposits {
							deposit_witnesses,
						}
						.into(),
					),
					epoch_index: epoch.epoch_index,
				})
				.await;
		}

		Ok(())
	}
}
