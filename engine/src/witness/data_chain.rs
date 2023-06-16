pub mod lag_safety;

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

	define_trait_alias!(pub trait Index: Step + PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Send + Sync + 'static);
	define_trait_alias!(pub trait Hash: PartialEq + Eq + Clone + Send + Sync + 'static);
	define_trait_alias!(pub trait Data: Send + Sync + 'static);
}

#[async_trait::async_trait]
pub trait DataChainSource: Send + Sync {
	type Index: aliases::Index;
	type Hash: aliases::Hash;
	type Data: aliases::Data;

	type Client: DataChainClient<Index = Self::Index, Hash = Self::Hash, Data = Self::Data>;
	type Stream: DataChainStream<Index = Self::Index, Hash = Self::Hash, Data = Self::Data>;

	async fn client_and_stream(&self) -> (Self::Client, Self::Stream);
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
