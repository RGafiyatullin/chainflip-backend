use std::sync::Arc;

use futures_core::stream::BoxStream;
use futures_util::StreamExt;
use utilities::task_scope::Scope;

use crate::{
	state_chain_observer::client::extrinsic_api::signed::SignedExtrinsicApi,
	witness::data_chain::DataChainClient,
};

use super::{DataChainSource, Header};

pub trait GetChainTrackingData<C: cf_chains::Chain> {
	fn get_chain_tracking_data(&self) -> C::TrackedData;
}

pub struct ChainDataDataChain<'env, C, I, ChainClient, SCC, DCS>
where
	C: cf_chains::Chain,
	I: 'static + Sync + Send,
	ChainClient: GetChainTrackingData<C>,
	SCC: SignedExtrinsicApi,
	DCS: DataChainSource,
{
	scope: Scope<'env, anyhow::Error>,
	chain_client: ChainClient,
	state_chain_client: Arc<SCC>,
	source: DCS,
}

impl<'env, C, I, ChainClient, SCC, DCS> ChainDataDataChain<'env, C, I, ChainClient, SCC, DCS>
where
	C: cf_chains::Chain,
	I: 'static + Sync + Send,
	ChainClient: GetChainTrackingData<C>,
	SCC: SignedExtrinsicApi + Send + Sync,
	DCS: DataChainSource,
{
	pub fn new(
		scope: &Scope<'env, anyhow::Error>,
		state_chain_client: Arc<SCC>,
		chain_client: ChainClient,
		source: DCS,
	) -> Self {
		Self { scope: scope.clone(), chain_client, state_chain_client, source }
	}
}

#[async_trait::async_trait]
impl<
		'env,
		C: cf_chains::Chain,
		I: 'static + Sync + Send,
		ChainClient: GetChainTrackingData<C>,
		SCC: SignedExtrinsicApi + Send + Sync,
		DCS: DataChainSource,
	> DataChainSource for ChainDataDataChain<'env, C, I, ChainClient, SCC, DCS>
{
	type Index = DCS::Index;
	type Hash = DCS::Hash;
	type Data = DCS::Data;

	type Stream = BoxStream<'static, Header<Self::Index, Self::Hash, Self::Data>>;

	async fn stream(&self) -> Self::Stream {
		let mut stream = self.data_chain_source.stream().await;

		let (data_sender, data_receiver) = async_channel::unbounded();

		let chain_client = self.chain_client.clone();

		// We can define some other trait that defines how chain data witnessing will be fetched.

		// We can use the same trait but a different struct, to ov

		self.scope.spawn(async move {
			utilities::loop_select!(
				if let Some(header) = stream.next() => {

					let chain_data = chain_client.get_chain_tracking_data(header.index).await;

					tracing::info!("Submitting chain data of {chain_data:?} for block {}", header.index);

					self.state_chain_client
					.submit_signed_extrinsic(state_chain_runtime::RuntimeCall::Witnesser(
						pallet_cf_witnesser::Call::witness_at_epoch {
							call: Box::new(pallet_cf_chain_tracking::Call::<state_chain_runtime::Runtime, I>::update_chain_state {
                                state: chain_data,
                            }),
							epoch_index: self.current_epoch.epoch_index,
						},
					))
					.await;
				} else break Ok(()),
			)
		});

		(client, Box::pin(data_receiver))
	}
}

#[cfg(test)]
mod tests {

	use crate::{
		state_chain_observer::client::mocks::MockStateChainClient,
		witness::data_chain::DataChainStream,
	};
	use futures_core::Stream;
	use futures_util::{stream, FutureExt};
	use utilities::task_scope::task_scope;

	use super::*;

	#[derive(Clone)]
	struct TestDataChainClient {}
	#[async_trait::async_trait]
	impl DataChainClient for TestDataChainClient {
		type Index = u32;
		type Hash = u32;
		type Data = ();

		async fn data_at_index(&self, _index: u32) -> Header<u32, u32, ()> {
			unreachable!()
		}
	}

	struct TestDataChainSource<Client, Stream> {
		client: Client,
		stream: Stream,
	}
	impl<Client, Stream> TestDataChainSource<Client, Stream> {
		fn new(client: Client, stream: Stream) -> Self {
			Self { client, stream }
		}
	}
	#[async_trait::async_trait]
	impl<
			Client: DataChainClient + Clone,
			Stream: DataChainStream<Index = Client::Index, Hash = Client::Hash, Data = Client::Data>
				+ Clone
				+ Sync,
		> DataChainSource for TestDataChainSource<Client, Stream>
	{
		type Index = Client::Index;
		type Hash = Client::Hash;
		type Data = Client::Data;

		type Stream = Stream;

		async fn stream(&self) -> Self::Stream {
			self.stream.clone()
		}
	}

	#[tokio::test]
	async fn no_items_no_submissions() {
		let blocks = 0..10;
		let headers = std::iter::zip(
			itertools::chain!(std::iter::once(None), blocks.clone().map(Some)),
			blocks,
		)
		.map(|(previous, index)| Header { index, hash: index, parent_hash: previous, data: () });

		task_scope(|scope| {
			async move {
				let mock_state_chain_client = Arc::new(MockStateChainClient::new());

				let chain_data = ChainDataDataChain::new(
					&scope,
					mock_state_chain_client,
					TestDataChainSource::new(TestDataChainClient {}, stream::iter(headers)),
				);

				let (_, stream) = chain_data.client_and_stream().await;

				Ok(())
			}
			.boxed()
		})
		.await
		.unwrap();
	}
}
