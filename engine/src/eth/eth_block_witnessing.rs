use std::{collections::BTreeSet, sync::Arc};

use async_trait::async_trait;
use cf_chains::Ethereum;
use futures::StreamExt;
use sp_core::H160;
use tokio::select;
use tracing::{info, info_span, trace, Instrument};

use super::{
	rpc::EthDualRpcClient, safe_dual_block_subscription_from, witnessing::AllWitnessers,
	EthNumberBloom,
};
use crate::{
	multisig::{ChainTag, PersistentKeyDB},
	try_with_logging,
	witnesser::{
		checkpointing::{
			get_witnesser_start_block_with_checkpointing, StartCheckpointing, WitnessedUntil,
		},
		epoch_witnesser, EpochStart,
	},
};

pub struct IngressAddressReceiverPairs {
	pub eth: (tokio::sync::mpsc::UnboundedReceiver<H160>, BTreeSet<H160>),
	pub flip: (tokio::sync::mpsc::UnboundedReceiver<H160>, BTreeSet<H160>),
	pub usdc: (tokio::sync::mpsc::UnboundedReceiver<H160>, BTreeSet<H160>),
}

#[async_trait]
pub trait BlockProcessor: Send {
	async fn process_block(
		&mut self,
		epoch: &EpochStart<Ethereum>,
		block: &EthNumberBloom,
	) -> anyhow::Result<()>;
}

pub async fn start(
	epoch_start_receiver: async_broadcast::Receiver<EpochStart<Ethereum>>,
	witnessers: AllWitnessers,
	eth_rpc: EthDualRpcClient,
	db: Arc<PersistentKeyDB>,
) -> Result<(), (async_broadcast::Receiver<EpochStart<Ethereum>>, IngressAddressReceiverPairs)> {
	epoch_witnesser::start(
		epoch_start_receiver,
		move |_| true,
		witnessers,
		move |mut end_witnessing_receiver, epoch, mut witnessers| {
			let eth_rpc = eth_rpc.clone();
			let db = db.clone();
			async move {
				let (from_block, witnessed_until_sender) =
					match get_witnesser_start_block_with_checkpointing::<cf_chains::Ethereum>(
						ChainTag::Ethereum,
						epoch.epoch_index,
						epoch.block_number,
						db,
					)
					.await
					.expect("Failed to start Eth witnesser checkpointing")
					{
						StartCheckpointing::Started((from_block, witnessed_until_sender)) =>
							(from_block, witnessed_until_sender),
						StartCheckpointing::AlreadyWitnessedEpoch =>
							return Result::<_, IngressAddressReceiverPairs>::Ok(witnessers),
					};

				// We need to return the receivers so we can restart the process while ensuring
				// we are still able to receive new ingress addresses to monitor.
				//
				// rustfmt chokes when formatting this macro.
				// See: https://github.com/rust-lang/rustfmt/issues/5404
				#[rustfmt::skip]
				macro_rules! try_with_logging_receivers {
					($exp:expr) => {
						try_with_logging!(
							$exp,
							IngressAddressReceiverPairs {
								eth: witnessers.eth_ingress.take_ingress_receiver_pair(),
								flip: witnessers.flip_ingress.take_ingress_receiver_pair(),
								usdc: witnessers.usdc_ingress.take_ingress_receiver_pair(),
							}
						)
					};
				}

				let mut block_stream = try_with_logging_receivers!(
					safe_dual_block_subscription_from(from_block, eth_rpc.clone()).await
				);

				let mut end_at_block = None;
				let mut current_block = from_block;

				loop {
					let block = select! {
						end_block = &mut end_witnessing_receiver => {
							end_at_block = Some(end_block.expect("end witnessing channel was dropped unexpectedly"));
							None
						}
						Some(block) = block_stream.next() => {
							current_block = block.block_number.as_u64();
							Some(block)
						}
					};

					if let Some(end_block) = end_at_block {
						if current_block >= end_block {
							info!("Eth block witnessers unsubscribe at block {end_block}");
							break
						}
					}

					if let Some(block) = block {
						let block_number = block.block_number.as_u64();
						trace!("Eth block witnessers are processing block {block_number}");

						try_with_logging_receivers!(futures::future::join_all([
							witnessers.key_manager.process_block(&epoch, &block),
							witnessers.stake_manager.process_block(&epoch, &block),
							witnessers.eth_ingress.process_block(&epoch, &block),
							witnessers.flip_ingress.process_block(&epoch, &block),
							witnessers.usdc_ingress.process_block(&epoch, &block),
						])
						.await
						.into_iter()
						.collect::<anyhow::Result<Vec<()>>>());

						witnessed_until_sender
							.send(WitnessedUntil { epoch_index: epoch.epoch_index, block_number })
							.await
							.unwrap();
					}
				}

				Ok(witnessers)
			}
		},
	)
	.instrument(info_span!("Eth-Block-Head-Witnesser"))
	.await
}
