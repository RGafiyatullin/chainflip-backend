use futures::{Future, Stream, StreamExt};
use jsonrpc_core_client::{RpcChannel, TypedClient, TypedSubscriptionStream, RpcResult};
use crate::p2p::{P2PNetworkClient, P2PMessage, P2PNetworkClientError, StatusCode};
use jsonrpc_core_client::transports::http::connect;
use tokio_compat_02::FutureExt;
use async_trait::async_trait;
use std::str;
use cf_p2p_rpc::P2pEvent;
use std::task::{Context, Poll};
use std::pin::Pin;

pub trait Base58 {
	fn to_base58(&self) -> String;
}

impl Base58 for () {
	fn to_base58(&self) -> String {
		"".to_string()
	}
}

struct GlueClient<'a> {
	url: &'a str,
}

impl<'a> GlueClient<'a> {
	pub fn new(url: &'a str) -> Self {
		GlueClient {
			url,
		}
	}
}

impl From<P2pEvent> for P2PMessage {
	fn from(p2p_event: P2pEvent) -> Self {
		match p2p_event {
			P2pEvent::Received(peer_id, msg) => {
				P2PMessage {
					sender_id: peer_id.parse().unwrap_or(0),
					data: msg,
				}
			}
			P2pEvent::PeerConnected(peer_id) => {
				P2PMessage {
					sender_id: peer_id.parse().unwrap_or(0),
					data: vec![],
				}
			}
			P2pEvent::PeerDisconnected(peer_id) => {
				P2PMessage {
					sender_id: peer_id.parse().unwrap_or(0),
					data: vec![],
				}
			}
		}
	}
}

struct GlueClientStream {
	// inner: TypedSubscriptionStream<P2pEvent>,
	inner: Pin<Box<dyn Stream<Item = RpcResult<P2pEvent>> + Send>>,
}

impl GlueClientStream {
	pub fn new(stream: TypedSubscriptionStream<P2pEvent>) -> Self {
		GlueClientStream {
			inner: Box::pin(stream),
		}
	}
}

impl Stream for GlueClientStream {
	type Item = P2PMessage;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = &mut *self;
		loop {
			match this.inner.poll_next_unpin(cx) {
				Poll::Ready(Some(result)) => {
					if let Ok(result) = result {
						return Poll::Ready(Some(result.into()))
					}
				}
				Poll::Ready(None) => return Poll::Ready(
					None
				),
				Poll::Pending => break
			}
		}

		Poll::Pending
	}
}

#[async_trait]
impl<'a, NodeId> P2PNetworkClient<NodeId, GlueClientStream> for GlueClient<'a>
	where NodeId: Base58 + Send + Sync
{
	async fn broadcast(&self, data: &[u8]) -> Result<StatusCode, P2PNetworkClientError> {
		let msg = str::from_utf8(data).map_err(|_| P2PNetworkClientError::Format)?;
		let client: P2PClient = FutureExt::compat(connect(self.url))
			.await
			.map_err(|_| P2PNetworkClientError::Rpc)?;

		client.broadcast(msg.to_string()).await.map_err(|_| P2PNetworkClientError::Rpc)
	}

	async fn send(&self, to: &NodeId, data: &[u8]) -> Result<StatusCode, P2PNetworkClientError> {
		let msg = str::from_utf8(data).map_err(|_| P2PNetworkClientError::Format)?;
		let client: P2PClient = FutureExt::compat(connect(self.url))
			.await
			.map_err(|_| P2PNetworkClientError::Rpc)?;

		client.send(to.to_base58(), msg.to_string()).await.map_err(|_| P2PNetworkClientError::Rpc)
	}

	async fn take_stream(&mut self) ->  Result<GlueClientStream, P2PNetworkClientError> {
		let client: P2PClient = FutureExt::compat(connect(self.url))
			.await
			.map_err(|_| P2PNetworkClientError::Rpc)?;

		let sub = client.subscribe_notifications()
			.map_err(|_| P2PNetworkClientError::Rpc)?;

		Ok(GlueClientStream::new(sub))
	}
}

#[derive(Clone)]
struct P2PClient {
	inner: TypedClient,
}

impl From<RpcChannel> for P2PClient {
	fn from(channel: RpcChannel) -> Self {
		P2PClient::new(channel.into())
	}
}

impl P2PClient {
	/// Creates a new `P2PClient`.
	pub fn new(sender: RpcChannel) -> Self {
		P2PClient {
			inner: sender.into(),
		}
	}
	/// Send a message to peer id returning a HTTP status code
	pub fn send(
		&self,
		peer_id: String,
		message: String,
	) -> impl Future<Output=RpcResult<u64>> {
		let args = (peer_id, message);
		self.inner.call_method("p2p_send", "u64", args)
	}

	/// Broadcast a message to the p2p network returning a HTTP status code
	/// impl Future<Output = RpcResult<R>>
	pub fn broadcast(&self, message: String) -> impl Future<Output=RpcResult<u64>> {
		let args = (message, );
		self.inner.call_method("p2p_broadcast", "u64", args)
	}

	// Subscribe to receive notifications
	pub fn subscribe_notifications(
		&self,
	) -> RpcResult<TypedSubscriptionStream<P2pEvent>>
	{
		let args_tuple = ();
		self.inner.subscribe(
			"cf_p2p_subscribeNotifications",
			args_tuple,
			"cf_p2p_notifications",
			"cf_p2p_unsubscribeNotifications",
			"RpcEvent",
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use jsonrpc_core_client::transports::http::connect;
	use tokio_compat_02::FutureExt;

	#[tokio::test]
	async fn should_work() {
		let mut glue_client = GlueClient::new(&"http://localhost:9933");
		//
		// if let Some(receiver) =
		// 	P2PNetworkClient::<ValidatorId>::take_receiver(&mut glue_client).await {
		//
		// }
	}
}
