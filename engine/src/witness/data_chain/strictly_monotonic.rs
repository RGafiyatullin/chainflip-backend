use std::task::Poll;

use futures::Stream;

use super::{DataChainSourceWithClient, DataChainStream};

#[pin_project::pin_project]
pub struct StrictlyMonotonicStream<St: DataChainStream> {
	#[pin]
	stream: St,
	last_output: Option<St::Index>,
}
impl<St: DataChainStream> Stream for StrictlyMonotonicStream<St> {
	type Item = St::Item;

	fn poll_next(
		self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> Poll<Option<Self::Item>> {
		let this = self.project();
		match this.stream.poll_next(cx) {
			Poll::Ready(Some(header)) => Poll::Ready(if Some(header.index) > *this.last_output {
				*this.last_output = Some(header.index);
				Some(header)
			} else {
				None
			}),
			poll_next => poll_next,
		}
	}
}

pub struct StrictlyMonotonic<DC: DataChainSourceWithClient> {
	data_chain_source: DC,
}
impl<DC: DataChainSourceWithClient> StrictlyMonotonic<DC> {
	pub fn new(data_chain_source: DC) -> Self {
		Self { data_chain_source }
	}
}
#[async_trait::async_trait]
impl<DC: DataChainSourceWithClient> DataChainSourceWithClient for StrictlyMonotonic<DC> {
	type Index = DC::Index;
	type Hash = DC::Hash;
	type Data = DC::Data;

	type Stream = StrictlyMonotonicStream<DC::Stream>;
	type Client = DC::Client;

	async fn stream_and_client(&self) -> (Self::Stream, Self::Client) {
		let (stream, client) = self.data_chain_source.stream_and_client().await;
		(StrictlyMonotonicStream { stream, last_output: None }, client)
	}
}
