use futures_core::Future;
use utilities::task_scope::Scope;

use crate::witness::{
	chunked_chain_source::{
		chunked_by_time::{builder::ChunkedByTimeBuilder, ChunkByTime},
		chunked_by_vault::{builder::ChunkedByVaultBuilder, ChunkByVault},
	},
	common::{ExternalChainSource, RuntimeHasChain},
	epoch_source::{EpochSource, VaultSource},
};

use super::{
	aliases, lag_safety::LagSafety, shared::SharedSource, strictly_monotonic::StrictlyMonotonic,
	then::Then, ChainSource, Header,
};

#[async_trait::async_trait]
pub trait ChainSourceExt: ChainSource {
	fn then<Output, Fut, ThenFn>(self, then_fn: ThenFn) -> Then<Self, ThenFn>
	where
		Self: Sized,
		Output: aliases::Data,
		Fut: Future<Output = Output> + Send + Sync,
		ThenFn: Fn(Header<Self::Index, Self::Hash, Self::Data>) -> Fut + Send + Sync + Clone,
	{
		Then::new(self, then_fn)
	}

	fn lag_safety(self, margin: usize) -> LagSafety<Self>
	where
		Self: Sized,
	{
		LagSafety::new(self, margin)
	}

	fn shared<'env>(self, scope: &Scope<'env, anyhow::Error>) -> SharedSource<Self>
	where
		Self: 'env + Sized,
		Self::Client: Clone,
		Self::Data: Clone,
	{
		SharedSource::new(self, scope)
	}

	fn strictly_monotonic(self) -> StrictlyMonotonic<Self>
	where
		Self: Sized,
	{
		StrictlyMonotonic::new(self)
	}

	async fn chunk_by_time<'b, 'env, StateChainClient: Send + Sync>(
		self,
		epochs: EpochSource<'b, 'env, StateChainClient, (), ()>,
	) -> ChunkedByTimeBuilder<ChunkByTime<Self>>
	where
		Self: ExternalChainSource + Sized,
	{
		ChunkedByTimeBuilder::new(ChunkByTime::new(self), epochs.into_stream().await.into_box())
	}

	async fn chunk_by_vault<'b, 'env, StateChainClient: Send + Sync>(
		self,
		vaults: VaultSource<'b, 'env, StateChainClient, Self::Chain>,
	) -> ChunkedByVaultBuilder<ChunkByVault<Self>>
	where
		Self: ExternalChainSource + Sized,
		state_chain_runtime::Runtime: RuntimeHasChain<Self::Chain>,
	{
		ChunkedByVaultBuilder::new(ChunkByVault::new(self), vaults.into_stream().await.into_box())
	}
}
impl<T: ChainSource> ChainSourceExt for T {}
