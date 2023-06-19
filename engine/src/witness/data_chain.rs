pub mod lag_safety;
pub mod strictly_monotonic;

use futures_core::Stream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header<Index, Hash, Data> {
	index: Index,
	hash: Hash,
	parent_hash: Option<Hash>,
	data: Data,
}

pub mod aliases {
	use std::iter::Step;

	macro_rules! define_trait_alias {
		(pub trait $name:ident: $($traits:tt)+) => {
			pub trait $name: $($traits)+ {}
			impl<T: $($traits)+> $name for T {}
		}
	}

	define_trait_alias!(pub trait Index: Step + PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Send + Sync + Unpin + 'static);
	define_trait_alias!(pub trait Hash: PartialEq + Eq + Clone + Send + Sync + Unpin + 'static);
	define_trait_alias!(pub trait Data: Send + Sync + Unpin + 'static);
}

#[async_trait::async_trait]
pub trait DataChainSource: Send + Sync {
	type Index: aliases::Index;
	type Hash: aliases::Hash;
	type Data: aliases::Data;

	type Stream: DataChainStream<Index = Self::Index, Hash = Self::Hash, Data = Self::Data>;

	async fn stream(&self) -> Self::Stream;
}

#[async_trait::async_trait]
pub trait DataChainSourceWithClient: Send + Sync {
	type Index: aliases::Index;
	type Hash: aliases::Hash;
	type Data: aliases::Data;

	type Stream: DataChainStream<Index = Self::Index, Hash = Self::Hash, Data = Self::Data>;
	type Client: DataChainClient<Index = Self::Index, Hash = Self::Hash, Data = Self::Data>;

	async fn stream_and_client(&self) -> (Self::Stream, Self::Client);
}

#[async_trait::async_trait]
impl<T: DataChainSourceWithClient> DataChainSource for T {
	type Index = T::Index;
	type Hash = T::Hash;
	type Data = T::Data;

	type Stream = T::Stream;

	async fn stream(&self) -> Self::Stream {
		self.stream_and_client().await.0
	}
}

#[async_trait::async_trait]
pub trait DataChainClient: Send + Sync {
	type Index: aliases::Index;
	type Hash: aliases::Hash;
	type Data: aliases::Data;

	async fn data_at_index(
		&self,
		index: Self::Index,
	) -> Header<Self::Index, Self::Hash, Self::Data>;
}

pub trait DataChainStream:
	Stream<Item = Header<Self::Index, Self::Hash, Self::Data>> + Unpin + Send
{
	type Index: aliases::Index;
	type Hash: aliases::Hash;
	type Data: aliases::Data;
}
impl<
		Index: aliases::Index,
		Hash: aliases::Hash,
		Data: aliases::Data,
		T: Stream<Item = Header<Index, Hash, Data>> + Unpin + Send,
	> DataChainStream for T
{
	type Index = Index;
	type Hash = Hash;
	type Data = Data;
}

#[cfg(test)]
pub mod test_utilities {
	use std::collections::BTreeMap;

	use crate::witness::data_chain::{DataChainClient, Header};

	use super::DataChainSourceWithClient;

	pub type TestIndex = u32;
	pub type TestHash = u32;
	pub type TestData = u32;

	pub fn generate_headers<It: Iterator<Item = TestIndex>>(
		it: It,
		hash_offset: TestHash,
	) -> Vec<Header<TestIndex, TestHash, TestData>> {
		it.map(move |index| Header {
			index,
			hash: index + hash_offset,
			parent_hash: index.checked_sub(1).map(|hash| hash + hash_offset),
			data: index,
		})
		.collect()
	}

	#[derive(Clone, Default)]
	pub struct TestDataChainClient {
		data_chain: BTreeMap<TestIndex, Header<TestIndex, TestHash, TestData>>,
	}
	#[async_trait::async_trait]
	impl DataChainClient for TestDataChainClient {
		type Index = TestIndex;
		type Hash = TestHash;
		type Data = TestData;

		async fn data_at_index(&self, index: u32) -> Header<TestIndex, TestHash, TestData> {
			*self.data_chain.get(&index).unwrap()
		}
	}

	pub struct TestDataChainSource {
		client: TestDataChainClient,
		stream: Vec<Header<TestIndex, TestHash, TestData>>,
	}
	impl TestDataChainSource {
		pub fn new<
			ClientData: IntoIterator<Item = Header<TestIndex, TestHash, TestData>>,
			StreamData: IntoIterator<Item = Header<TestIndex, TestHash, TestData>>,
		>(
			client_data: ClientData,
			stream_data: StreamData,
		) -> Self {
			Self {
				client: TestDataChainClient {
					data_chain: client_data
						.into_iter()
						.map(|header| (header.index, header))
						.collect(),
				},
				stream: stream_data.into_iter().collect::<Vec<_>>(),
			}
		}
	}
	#[async_trait::async_trait]
	impl DataChainSourceWithClient for TestDataChainSource {
		type Index = TestIndex;
		type Hash = TestHash;
		type Data = TestData;

		type Stream =
			futures::stream::Iter<std::vec::IntoIter<Header<TestIndex, TestHash, TestData>>>;
		type Client = TestDataChainClient;

		async fn stream_and_client(&self) -> (Self::Stream, Self::Client) {
			(futures::stream::iter(self.stream.clone()), self.client.clone())
		}
	}
}
