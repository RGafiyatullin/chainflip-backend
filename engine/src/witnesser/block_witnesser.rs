use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;
use cf_primitives::EpochIndex;
use futures::Stream;

use crate::multisig::{HasChainTag, PersistentKeyDB};

use super::{
	checkpointing::{
		get_witnesser_start_block_with_checkpointing, StartCheckpointing, WitnessedUntil,
	},
	epoch_witnesser::{EpochWitnesser, EpochWitnesserGenerator, WitnesserAndStream},
	ChainBlockNumber, EpochStart,
};

type BlockNumber<Witnesser> = ChainBlockNumber<<Witnesser as EpochWitnesser>::Chain>;

// MAXIM: deduplicate this
pub trait HasBlockNumber2 {
	type BlockNumber;

	fn block_number(&self) -> Self::BlockNumber;
}

#[async_trait]
pub trait BlockWitnesserProcessor: Send + Sync + 'static {
	type Chain: cf_chains::Chain + HasChainTag;
	type Block: Send + HasBlockNumber2<BlockNumber = ChainBlockNumber<Self::Chain>>;
	type StaticState: Send;

	async fn process_block(
		&mut self,
		block: Self::Block,
		state: &mut Self::StaticState,
	) -> anyhow::Result<()>;
}

pub struct BlockWitnesser<W>
where
	W: BlockWitnesserProcessor,
{
	pub witnesser: W,
	pub epoch_index: EpochIndex,
	pub witnessed_until_sender: tokio::sync::mpsc::Sender<WitnessedUntil>,
	pub last_processed_block: ChainBlockNumber<W::Chain>,
}

#[async_trait]
impl<W> EpochWitnesser for BlockWitnesser<W>
where
	W: BlockWitnesserProcessor,
{
	type Chain = W::Chain;
	type Data = W::Block;
	type StaticState = W::StaticState;

	async fn do_witness(
		&mut self,
		block: W::Block,
		state: &mut Self::StaticState,
	) -> anyhow::Result<()> {
		let block_number = block.block_number();

		self.witnesser.process_block(block, state).await?;

		self.last_processed_block = block_number;

		self.witnessed_until_sender
			.send(WitnessedUntil {
				epoch_index: self.epoch_index,
				block_number: block_number.into(),
			})
			.await
			.unwrap();

		Ok(())
	}

	fn should_finish(&self, last_block_in_epoch: BlockNumber<Self>) -> bool {
		self.last_processed_block >= last_block_in_epoch
	}
}

// --------------------------------------------------------------------------

type BlockStream<Block> = Pin<Box<dyn Stream<Item = anyhow::Result<Block>> + Send + 'static>>;

#[async_trait]
pub trait BlockWitnesserGeneratorTrait: Send {
	type Witnesser: BlockWitnesserProcessor;

	fn create_witnesser(epoch: EpochStart<<Self::Witnesser as BlockWitnesserProcessor>::Chain>) -> Self::Witnesser;

	async fn get_block_stream(
		&mut self,
		from_block: ChainBlockNumber<<Self::Witnesser as BlockWitnesserProcessor>::Chain>,
	) -> anyhow::Result<BlockStream<<Self::Witnesser as BlockWitnesserProcessor>::Block>>;
}

pub struct BlockWitnesserGenerator<W>
where
	W: BlockWitnesserGeneratorTrait,
{
	pub generator: W,
	pub db: Arc<PersistentKeyDB>,
}

#[async_trait]
impl<W> EpochWitnesserGenerator for BlockWitnesserGenerator<W>
where
	W: BlockWitnesserGeneratorTrait,
	<<<W::Witnesser as BlockWitnesserProcessor>::Chain as cf_chains::Chain>::ChainBlockNumber as TryFrom<u64>>::Error: std::fmt::Debug
{
	type Witnesser = BlockWitnesser<W::Witnesser>;

	async fn init(
		&mut self,
		epoch: EpochStart<<W::Witnesser as BlockWitnesserProcessor>::Chain>,
	) -> anyhow::Result<Option<WitnesserAndStream<BlockWitnesser<W::Witnesser>>>> {
		let (from_block, witnessed_until_sender) =
			match get_witnesser_start_block_with_checkpointing::<
				<W::Witnesser as BlockWitnesserProcessor>::Chain,
			>(epoch.epoch_index, epoch.block_number, self.db.clone())
			.await
			.expect("Failed to start Eth witnesser checkpointing")
			{
				StartCheckpointing::Started((from_block, witnessed_until_sender)) =>
					(from_block, witnessed_until_sender),
				StartCheckpointing::AlreadyWitnessedEpoch => return Ok(None),
			};

		let block_stream = self.generator.get_block_stream(from_block).await?;

			let witnesser = BlockWitnesser {
				epoch_index: epoch.epoch_index,
				witnesser: W::create_witnesser(epoch),
				witnessed_until_sender,
				last_processed_block: from_block,
			};

			
	Ok(Some((witnesser, block_stream)))

	}

	fn should_process_historical_epochs() -> bool {
		true
	}
}
