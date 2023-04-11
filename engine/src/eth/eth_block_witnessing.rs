use std::{pin::Pin, sync::Arc, time::Duration};

use async_trait::async_trait;
use cf_chains::Ethereum;
use futures::Stream;
use tokio::sync::Mutex;
use tracing::{info_span, Instrument};

use super::{
	rpc::EthDualRpcClient, safe_dual_block_subscription_from, witnessing::AllWitnessers,
	EthNumberBloom,
};
use crate::{
	constants::{BLOCK_PULL_TIMEOUT_MULTIPLIER, ETH_AVERAGE_BLOCK_TIME_SECONDS},
	multisig::PersistentKeyDB,
	stream_utils::EngineStreamExt,
	witnesser::{
		block_witnesser::{
			BlockWitnesserGenerator, BlockWitnesserGeneratorTrait, BlockWitnesserProcessor,
			HasBlockNumber2,
		},
		epoch_witnesser::start_epoch_witnesser,
		ChainBlockNumber, EpochStart,
	},
};

#[async_trait]
pub trait BlockProcessor: Send {
	async fn process_block(
		&mut self,
		epoch: &EpochStart<Ethereum>,
		block: &EthNumberBloom,
	) -> anyhow::Result<()>;
}

struct EthBlockWitnesser {
	epoch: EpochStart<Ethereum>,
}

struct EthBlockWitnesserGenerator {
	eth_dual_rpc: EthDualRpcClient,
}

pub async fn start(
	epoch_start_receiver: Arc<Mutex<async_broadcast::Receiver<EpochStart<Ethereum>>>>,
	witnessers: AllWitnessers,
	eth_dual_rpc: EthDualRpcClient,
	db: Arc<PersistentKeyDB>,
) -> Result<(), ()> {
	start_epoch_witnesser(
		epoch_start_receiver,
		BlockWitnesserGenerator { db, generator: EthBlockWitnesserGenerator { eth_dual_rpc } },
		witnessers,
	)
	.instrument(info_span!("Eth-Block-Head-Witnesser"))
	.await
}

// -------------------------------------------------------------

impl HasBlockNumber2 for EthNumberBloom {
	type BlockNumber = ChainBlockNumber<Ethereum>;

	fn block_number(&self) -> Self::BlockNumber {
		self.block_number.as_u64()
	}
}

#[async_trait]
impl BlockWitnesserProcessor for EthBlockWitnesser {
	type Chain = Ethereum;
	type Block = EthNumberBloom;
	type StaticState = AllWitnessers;

	async fn process_block(
		&mut self,
		block: Self::Block,
		witnessers: &mut AllWitnessers,
	) -> anyhow::Result<()> {
		tracing::trace!("Eth block witnessers are processing block {}", block.block_number);

		futures::future::join_all([
			witnessers.key_manager.process_block(&self.epoch, &block),
			witnessers.stake_manager.process_block(&self.epoch, &block),
			witnessers.eth_ingress.process_block(&self.epoch, &block),
			witnessers.flip_ingress.process_block(&self.epoch, &block),
			witnessers.usdc_ingress.process_block(&self.epoch, &block),
		])
		.await
		.into_iter()
		.collect::<anyhow::Result<Vec<()>>>()
		.map_err(|err| {
			tracing::error!("Eth witnesser failed to process block: {err}");
			err
		})?;

		Ok(())
	}
}

type BlockStream = Pin<Box<dyn Stream<Item = anyhow::Result<EthNumberBloom>> + Send + 'static>>;

#[async_trait]
impl BlockWitnesserGeneratorTrait for EthBlockWitnesserGenerator {
	type Witnesser = EthBlockWitnesser;

	fn create_witnesser(epoch: EpochStart<Ethereum>) -> Self::Witnesser {
		EthBlockWitnesser { epoch }
	}

	async fn get_block_stream(
		&mut self,
		from_block: ChainBlockNumber<Ethereum>,
	) -> anyhow::Result<BlockStream> {
		let block_stream = safe_dual_block_subscription_from(from_block, self.eth_dual_rpc.clone())
			.await
			.map_err(|err| {
				tracing::error!("Subscription error: {err}");
				err
			})?
			.timeout_after(Duration::from_secs(
				ETH_AVERAGE_BLOCK_TIME_SECONDS * BLOCK_PULL_TIMEOUT_MULTIPLIER,
			));

		Ok(Box::pin(block_stream))
	}
}
