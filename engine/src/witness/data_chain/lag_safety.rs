use std::{collections::VecDeque, iter::Step};

use futures_core::stream::BoxStream;
use futures_util::StreamExt;
use utilities::task_scope::Scope;

use crate::witness::data_chain::DataChainClient;

use super::{DataChainSource, Header};

pub struct LagSafetyDataChain<'env, DC: DataChainSource> {
	scope: Scope<'env, anyhow::Error>,
	data_chain_source: DC,
	margin: usize,
}
impl<'env, DC: DataChainSource> LagSafetyDataChain<'env, DC> {
	pub fn new(scope: &Scope<'env, anyhow::Error>, margin: usize, data_chain_source: DC) -> Self {
		Self { scope: scope.clone(), data_chain_source, margin }
	}
}
#[async_trait::async_trait]
impl<'env, DC: DataChainSource> DataChainSource for LagSafetyDataChain<'env, DC>
where
	DC::Client: Clone + 'env,
	DC::Stream: 'env,
{
	type Index = DC::Index;
	type Hash = DC::Hash;
	type Data = DC::Data;

	type Client = DC::Client;
	type Stream = BoxStream<'static, Header<Self::Index, Self::Hash, Self::Data>>;

	async fn client_and_stream(&self) -> (Self::Client, Self::Stream) {
		let (client, mut stream) = self.data_chain_source.client_and_stream().await;
		let margin = self.margin;

		let (data_sender, data_receiver) = async_channel::unbounded();

		let lag_client = client.clone(); // TODO: Add delay into Client too

		self.scope.spawn(async move {
			let mut latest_safe_index: Option<<Self as DataChainSource>::Index> = None;
			let mut unsafe_cache = VecDeque::<Header<<Self as DataChainSource>::Index, <Self as DataChainSource>::Hash, <Self as DataChainSource>::Data>>::new();
			utilities::loop_select!(
				if let Some(header) = stream.next() => {
					if unsafe_cache.back().map_or(false, |last_header| Some(&last_header.hash) != header.parent_hash.as_ref() || Step::forward_checked(last_header.index, 1) != Some(header.index)) {
						unsafe_cache.clear();
					}
					let header_index = header.index;
					if Some(header_index) > latest_safe_index {
						unsafe_cache.push_back(header);
						if let Some(new_latest_safe_index) = Step::backward_checked(header_index, margin).filter(|safe_index| Some(*safe_index) > latest_safe_index) {
							latest_safe_index = Some(new_latest_safe_index);
							let _result = data_sender.send(if unsafe_cache.len() > margin {
								assert_eq!(unsafe_cache.len() - 1, margin);
								unsafe_cache.pop_front().unwrap()
							} else {
								client.data_at_index(new_latest_safe_index).await
							}).await;
						}
					}

				} else break Ok(()),
			)
		});

		(lag_client, Box::pin(data_receiver))
	}
}

#[cfg(test)]
mod test {
	use futures_util::{stream, FutureExt, StreamExt};
	use itertools::Itertools;
	use utilities::task_scope;

	use crate::witness::data_chain::{DataChainClient, DataChainSource, DataChainStream, Header};

	use super::LagSafetyDataChain;

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

		type Client = Client;
		type Stream = Stream;

		async fn client_and_stream(&self) -> (Self::Client, Self::Stream) {
			(self.client.clone(), self.stream.clone())
		}
	}

	#[tokio::test]
	async fn maintains_good_stream() {
		let blocks = 0..10;
		let headers = std::iter::zip(
			itertools::chain!(std::iter::once(None), blocks.clone().map(Some)),
			blocks,
		)
		.map(|(previous, index)| Header { index, hash: index, parent_hash: previous, data: () });

		task_scope::task_scope(|scope| {
			async {
				let lag_safety = LagSafetyDataChain::new(
					scope,
					5,
					TestDataChainSource::new(TestDataChainClient {}, stream::iter(headers)),
				);

				let (_, stream) = lag_safety.client_and_stream().await;

				assert_eq!(
					stream.collect::<Vec<_>>().await,
					(0..5u32)
						.map(|i| Header {
							index: i,
							hash: i,
							data: (),
							parent_hash: i.checked_sub(1)
						})
						.collect_vec()
				);

				Ok(())
			}
			.boxed()
		})
		.await
		.unwrap();
	}
}
