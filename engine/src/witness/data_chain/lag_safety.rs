use std::{collections::VecDeque, iter::Step};

use futures_util::StreamExt;
use utilities::task_scope::Scope;

use crate::witness::data_chain::DataChainClient;

use super::{DataChainSourceWithClient, Header};

pub struct LagSafety<'env, DC: DataChainSourceWithClient> {
	scope: Scope<'env, anyhow::Error>,
	data_chain_source: DC,
	margin: usize,
}
impl<'env, DC: DataChainSourceWithClient> LagSafety<'env, DC> {
	pub fn new(scope: &Scope<'env, anyhow::Error>, margin: usize, data_chain_source: DC) -> Self {
		Self { scope: scope.clone(), data_chain_source, margin }
	}
}
#[async_trait::async_trait]
impl<'env, DC: DataChainSourceWithClient> DataChainSourceWithClient for LagSafety<'env, DC>
where
	DC::Client: Clone + 'env,
	DC::Stream: 'env,
{
	type Index = DC::Index;
	type Hash = DC::Hash;
	type Data = DC::Data;

	type Stream = async_channel::Receiver<Header<Self::Index, Self::Hash, Self::Data>>;
	type Client = DC::Client;

	async fn stream_and_client(&self) -> (Self::Stream, Self::Client) {
		let (mut stream, client) = self.data_chain_source.stream_and_client().await;
		let margin = self.margin;
		let (data_sender, data_receiver) = async_channel::unbounded();

		self.scope.spawn({
			let client = client.clone(); // TODO: Add delay into Client too
			async move {
				let mut unsafe_cache = VecDeque::<Header<<Self as DataChainSourceWithClient>::Index, <Self as DataChainSourceWithClient>::Hash, <Self as DataChainSourceWithClient>::Data>>::new();
				utilities::loop_select!(
					if let Some(header) = stream.next() => {
						let header_index = header.index;
						if unsafe_cache.back().map_or(false, |last_header| Some(&last_header.hash) != header.parent_hash.as_ref() || Step::forward_checked(last_header.index, 1) != Some(header.index)) {
							unsafe_cache.clear();
						}
						unsafe_cache.push_back(header);
						if let Some(next_output_index) = Step::backward_checked(header_index, margin) {
							let _result = data_sender.send(if unsafe_cache.len() > margin {
								assert_eq!(unsafe_cache.len() - 1, margin);
								unsafe_cache.pop_front().unwrap()
							} else {
								// We don't check sequence of hashes and assume due to order of requests it will be safe (even though this is not true)
								client.data_at_index(next_output_index).await
							}).await;
						}
					} else break Ok(()),
				)
			}
		});

		(data_receiver, client)
	}
}

#[cfg(test)]
mod test {
	use futures_util::{FutureExt, StreamExt};
	use utilities::task_scope;

	use crate::witness::data_chain::{
		test_utilities::{generate_headers, TestDataChainSource},
		DataChainSourceWithClient,
	};

	use super::LagSafety;

	#[tokio::test]
	async fn maintains_good_stream() {
		task_scope::task_scope(|scope| {
			async {
				const MARGIN: usize = 5;

				let headers = generate_headers(0..10, 0);

				let lag_safety = LagSafety::new(
					scope,
					MARGIN,
					TestDataChainSource::new(std::iter::empty(), headers.clone()),
				);

				let (stream, _) = lag_safety.stream_and_client().await;

				assert!(Iterator::eq(
					stream.collect::<Vec<_>>().await.iter(),
					headers.iter().take(headers.len() - MARGIN)
				));

				Ok(())
			}
			.boxed()
		})
		.await
		.unwrap();
	}
}
