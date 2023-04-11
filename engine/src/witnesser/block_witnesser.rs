use async_trait::async_trait;

use super::{epoch_witnesser::EpochWitnesser, ChainBlockNumber};

type BlockNumber<Witnesser> = ChainBlockNumber<<Witnesser as EpochWitnesser>::Chain>;

// MAXIM: deduplicate this
pub trait HasBlockNumber2 {
	type BlockNumber;

	fn block_number(&self) -> Self::BlockNumber;
}

#[async_trait]
pub trait BlockWitnesserProcessor: Send + Sync + 'static {
	type Chain: cf_chains::Chain;
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

		Ok(())
	}

	fn should_finish(&self, last_block_in_epoch: BlockNumber<Self>) -> bool {
		self.last_processed_block >= last_block_in_epoch
	}
}

// --------------------------------------------------------------------------
