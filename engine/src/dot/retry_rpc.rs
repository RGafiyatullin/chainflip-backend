use crate::{
	dot::PolkadotConfig,
	witness::chain_source::{ChainClient, Header},
};
use cf_chains::{
	dot::{PolkadotHash, RuntimeVersion},
	Polkadot,
};
use cf_primitives::PolkadotBlockNumber;
use core::time::Duration;
use futures_core::Stream;
use std::pin::Pin;
use subxt::{
	config::Header as SubxtHeader,
	events::Events,
	rpc::types::{ChainBlockExtrinsic, ChainBlockResponse},
};
use utilities::task_scope::Scope;

use crate::rpc_retrier::RpcRetrierClient;

use super::rpc::{DotRpcClient, PolkadotHeader};

use crate::dot::rpc::DotRpcApi;

pub struct DotRetryRpcClient {
	retry_client: RpcRetrierClient<DotRpcClient>,
}

const POLKADOT_RPC_TIMEOUT: Duration = Duration::from_millis(1000);
const MAX_CONCURRENT_SUBMISSIONS: u32 = 100;

impl DotRetryRpcClient {
	pub fn new(scope: &Scope<'_, anyhow::Error>, dot_client: DotRpcClient) -> Self {
		Self {
			retry_client: RpcRetrierClient::new(
				scope,
				dot_client,
				POLKADOT_RPC_TIMEOUT,
				MAX_CONCURRENT_SUBMISSIONS,
			),
		}
	}
}

#[async_trait::async_trait]
pub trait DotRetryRpcApi {
	async fn block_hash(&self, block_number: PolkadotBlockNumber) -> Option<PolkadotHash>;

	async fn block(&self, block_hash: PolkadotHash) -> Option<ChainBlockResponse<PolkadotConfig>>;

	async fn extrinsics(&self, block_hash: PolkadotHash) -> Option<Vec<ChainBlockExtrinsic>>;

	async fn events(&self, block_hash: PolkadotHash) -> Option<Events<PolkadotConfig>>;

	async fn current_runtime_version(&self) -> RuntimeVersion;

	async fn submit_raw_encoded_extrinsic(&self, encoded_bytes: Vec<u8>) -> PolkadotHash;
}

#[async_trait::async_trait]
impl DotRetryRpcApi for DotRetryRpcClient {
	async fn block_hash(&self, block_number: PolkadotBlockNumber) -> Option<PolkadotHash> {
		self.retry_client
			.request(Box::pin(move |client| {
				#[allow(clippy::redundant_async_block)]
				Box::pin(async move { client.block_hash(block_number).await })
			}))
			.await
	}

	async fn block(&self, block_hash: PolkadotHash) -> Option<ChainBlockResponse<PolkadotConfig>> {
		self.retry_client
			.request(Box::pin(move |client| {
				#[allow(clippy::redundant_async_block)]
				Box::pin(async move { client.block(block_hash).await })
			}))
			.await
	}

	async fn extrinsics(&self, block_hash: PolkadotHash) -> Option<Vec<ChainBlockExtrinsic>> {
		self.retry_client
			.request(Box::pin(move |client| {
				#[allow(clippy::redundant_async_block)]
				Box::pin(async move { client.extrinsics(block_hash).await })
			}))
			.await
	}

	async fn events(&self, block_hash: PolkadotHash) -> Option<Events<PolkadotConfig>> {
		self.retry_client
			.request(Box::pin(move |client| {
				#[allow(clippy::redundant_async_block)]
				Box::pin(async move { client.events(block_hash).await })
			}))
			.await
	}

	async fn current_runtime_version(&self) -> RuntimeVersion {
		self.retry_client
			.request(Box::pin(move |client| {
				#[allow(clippy::redundant_async_block)]
				Box::pin(async move { client.current_runtime_version().await })
			}))
			.await
	}

	async fn submit_raw_encoded_extrinsic(&self, encoded_bytes: Vec<u8>) -> PolkadotHash {
		self.retry_client
			.request(Box::pin(move |client| {
				let encoded_bytes = encoded_bytes.clone();
				#[allow(clippy::redundant_async_block)]
				Box::pin(async move { client.submit_raw_encoded_extrinsic(encoded_bytes).await })
			}))
			.await
	}
}

#[async_trait::async_trait]
pub trait DotRetrySubscribeApi {
	async fn subscribe_best_heads(
		&self,
	) -> Pin<Box<dyn Stream<Item = anyhow::Result<PolkadotHeader>> + Send>>;

	async fn subscribe_finalized_heads(
		&self,
	) -> Pin<Box<dyn Stream<Item = anyhow::Result<PolkadotHeader>> + Send>>;
}

use crate::dot::rpc::DotSubscribeApi;

#[async_trait::async_trait]
impl DotRetrySubscribeApi for DotRetryRpcClient {
	async fn subscribe_best_heads(
		&self,
	) -> Pin<Box<dyn Stream<Item = anyhow::Result<PolkadotHeader>> + Send>> {
		self.retry_client
			.request(Box::pin(move |client| {
				#[allow(clippy::redundant_async_block)]
				Box::pin(async move { client.subscribe_best_heads().await })
			}))
			.await
	}

	async fn subscribe_finalized_heads(
		&self,
	) -> Pin<Box<dyn Stream<Item = anyhow::Result<PolkadotHeader>> + Send>> {
		self.retry_client
			.request(Box::pin(move |client| {
				#[allow(clippy::redundant_async_block)]
				Box::pin(async move { client.subscribe_finalized_heads().await })
			}))
			.await
	}
}

#[async_trait::async_trait]
impl ChainClient for DotRetryRpcClient {
	type Index = <Polkadot as cf_chains::Chain>::ChainBlockNumber;
	type Hash = PolkadotHash;
	type Data = Events<PolkadotConfig>;

	async fn header_at_index(
		&self,
		index: Self::Index,
	) -> Header<Self::Index, Self::Hash, Self::Data> {
		self.retry_client
			.request(Box::pin(move |client| {
				#[allow(clippy::redundant_async_block)]
				Box::pin(async move {
					let block_hash = client
						.block_hash(index)
						.await?
						.ok_or(anyhow::anyhow!("No block hash found for index {index}"))?;
					let header = client
						.block(block_hash)
						.await?
						.ok_or(anyhow::anyhow!("No block found for block hash {block_hash}"))?
						.block
						.header;

					assert_eq!(index, header.number);

					let events = client
						.events(block_hash)
						.await?
						.ok_or(anyhow::anyhow!("No events found for block hash {block_hash}"))?;
					Ok(Header {
						index,
						hash: header.hash(),
						parent_hash: Some(header.parent_hash),
						data: events,
					})
				})
			}))
			.await
	}
}

#[cfg(test)]
mod tests {
	use futures_util::FutureExt;

	use utilities::task_scope::task_scope;

	use super::*;

	#[tokio::test]
	#[ignore = "Requires network connection and will last forever with failing extrinsic submission"]
	async fn my_test() {
		task_scope(|scope| {
			async move {
				let dot_client = DotRpcClient::new("ws://127.0.0.1:9945").await.unwrap();
				let dot_retry_rpc_client = DotRetryRpcClient::new(scope, dot_client);

				let hash = dot_retry_rpc_client.block_hash(1).await.unwrap();
				println!("Block hash: {}", hash);

				let extrinsics = dot_retry_rpc_client.extrinsics(hash).await.unwrap();
				println!("extrinsics: {:?}", extrinsics);

				let events = dot_retry_rpc_client.events(hash).await;
				println!("Events: {:?}", events);

				let runtime_version = dot_retry_rpc_client.current_runtime_version().await;
				println!("Runtime version: {:?}", runtime_version);

				let hash = dot_retry_rpc_client
					.submit_raw_encoded_extrinsic(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
					.await;
				println!("Extrinsic hash: {}", hash);

				Ok(())
			}
			.boxed()
		})
		.await
		.unwrap();
	}
}
