use std::{
	collections::BTreeMap,
	pin::Pin,
	sync::{atomic, Arc},
	task::{ready, Context, Poll},
};

use web3::transports::ws::compat::{TcpStream, TlsStream};

use futures::{
	channel::{mpsc, oneshot},
	pin_mut, select, AsyncRead, AsyncWrite, Future, FutureExt, Stream, StreamExt,
};
use jsonrpc_core as rpc;
use soketto::{
	connection,
	handshake::{Client, ServerResponse},
};
use url::Url;
use web3::{
	api::SubscriptionId,
	error::{self, TransportError},
	helpers,
	transports::ws::compat,
	DuplexTransport, RequestId,
};

type SingleResult = error::Result<rpc::Value>;
type BatchResult = error::Result<Vec<SingleResult>>;
type Subscription = mpsc::UnboundedSender<rpc::Value>;
type Pending = oneshot::Sender<BatchResult>;

use super::{rpc::EthTransport, TransportProtocol};

#[derive(Clone, Debug)]
pub struct WebSocket {
	id: Arc<atomic::AtomicUsize>,
	requests: mpsc::UnboundedSender<TransportMessage>,
}

#[derive(Debug)]
enum TransportMessage {
	Request { id: RequestId, request: String, sender: oneshot::Sender<BatchResult> },
	Subscribe { id: SubscriptionId, sink: mpsc::UnboundedSender<rpc::Value> },
	Unsubscribe { id: SubscriptionId },
}

impl WebSocket {
	pub async fn new(ws_endpont: &str) -> anyhow::Result<Self> {
		let id = Arc::new(atomic::AtomicUsize::new(1));
		let task = WsServerTask::new(ws_endpont).await?;
		let (request_sender, request_receiver) = mpsc::unbounded();
		tokio::spawn(task.into_task(request_receiver));
		Ok(WebSocket { id, requests: request_sender })
	}

	fn send(&self, msg: TransportMessage) -> error::Result {
		println!("send");
		self.requests.unbounded_send(msg).map_err(dropped_err)
	}

	fn send_request(
		&self,
		id: RequestId,
		request: rpc::Request,
	) -> error::Result<oneshot::Receiver<BatchResult>> {
		let request = helpers::to_string(&request);
		println!("[{}] Calling: {}", id, request);
		let (sender, receiver) = oneshot::channel();
		self.send(TransportMessage::Request { id, request, sender })?;
		Ok(receiver)
	}
}

impl EthTransport for WebSocket {
	fn transport_protocol() -> super::TransportProtocol {
		TransportProtocol::Ws
	}
}

impl DuplexTransport for WebSocket {
	type NotificationStream = mpsc::UnboundedReceiver<rpc::Value>;

	fn subscribe(&self, id: SubscriptionId) -> error::Result<Self::NotificationStream> {
		let (sink, stream) = mpsc::unbounded();
		println!("Subscribing, subscription {:?},", &id);
		self.send(TransportMessage::Subscribe { id, sink })?;
		Ok(stream)
	}

	fn unsubscribe(&self, id: SubscriptionId) -> error::Result {
		println!("Unsubscribing, subscription {:?}", id);
		self.send(TransportMessage::Unsubscribe { id })?;
		Ok(())
	}
}

impl web3::Transport for WebSocket {
	type Out = MyResponse<rpc::Value, fn(BatchResult) -> SingleResult>;

	fn prepare(&self, method: &str, params: Vec<rpc::Value>) -> (web3::RequestId, rpc::Call) {
		println!("Transport::prepare");
		let id = self.id.fetch_add(1, atomic::Ordering::AcqRel);
		let request = helpers::build_request(id, method, params);

		(id, request)
	}

	fn send(&self, id: web3::RequestId, request: rpc::Call) -> Self::Out {
		println!("Transport::send");
		let response = self.send_request(id, rpc::Request::Single(request));
		MyResponse::new(response, batch_to_single)
	}
}

fn batch_to_single(response: BatchResult) -> SingleResult {
	match response?.into_iter().next() {
		Some(res) => res,
		None => Err(error::Error::InvalidResponse("Expected single, got batch.".into())),
	}
}

enum ResponseState {
	Receiver(Option<error::Result<oneshot::Receiver<BatchResult>>>),
	Waiting(oneshot::Receiver<BatchResult>),
}

/// A WS resonse wrapper.
pub struct MyResponse<R, T> {
	extract: T,
	state: ResponseState,
	_data: std::marker::PhantomData<R>,
}

impl<R, T> MyResponse<R, T> {
	fn new(response: error::Result<oneshot::Receiver<BatchResult>>, extract: T) -> Self {
		Self { extract, state: ResponseState::Receiver(Some(response)), _data: Default::default() }
	}
}

impl<R, T> Future for MyResponse<R, T>
where
	R: Unpin + 'static,
	T: Fn(BatchResult) -> error::Result<R> + Unpin + 'static,
{
	type Output = error::Result<R>;
	fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		loop {
			match self.state {
				ResponseState::Receiver(ref mut res) => {
					let receiver = res.take().expect("Receiver state is active only once; qed")?;
					self.state = ResponseState::Waiting(receiver)
				},
				ResponseState::Waiting(ref mut future) => {
					let response = ready!(future.poll_unpin(cx)).map_err(dropped_err)?;
					return Poll::Ready((self.extract)(response))
				},
			}
		}
	}
}

fn dropped_err<T>(_: T) -> error::Error {
	error::Error::Transport(TransportError::Message(
		"Cannot send request. Internal task finished.".into(),
	))
}

struct WsServerTask {
	sender: connection::Sender<MaybeTlsStream<TcpStream, TlsStream>>,
	receiver: connection::Receiver<MaybeTlsStream<TcpStream, TlsStream>>,
	subscriptions: BTreeMap<SubscriptionId, Subscription>,
	pending: BTreeMap<RequestId, Pending>,
}

impl WsServerTask {
	pub async fn new(url: &str) -> error::Result<Self> {
		let url = Url::parse(url).unwrap();

		let scheme = match url.scheme() {
			s if s == "ws" || s == "wss" => s,
			s =>
				return Err(error::Error::Transport(TransportError::Message(format!(
					"Wrong scheme: {}",
					s
				)))),
		};

		let host = match url.host_str() {
			Some(s) => s,
			None =>
				return Err(error::Error::Transport(TransportError::Message(
					"Wrong host name".to_string(),
				))),
		};

		let port = url.port().unwrap_or(if scheme == "ws" { 80 } else { 443 });

		let addrs = format!("{}:{}", host, port);

		println!("Connecting TcpStream with address: {}", addrs);
		let stream = tokio::net::TcpStream::connect(addrs).await?;

		let socket = if scheme == "wss" {
			let stream = async_native_tls::connect(host, stream).await?;
			let stream = compat::compat(stream);
			MaybeTlsStream::Tls(stream)
		} else {
			let stream = compat::compat(stream);
			MaybeTlsStream::Plain(stream)
		};

		let resource = match url.query() {
			Some(q) => format!("{}?{}", url.path(), q),
			None => url.path().to_owned(),
		};

		println!("Connecting websocket client with host: {} and resource: {}", host, resource);

		let mut client = Client::new(socket, host, &resource);
		let maybe_encoded = url.password().map(|password| {
			use headers::authorization::{Authorization, Credentials};
			Authorization::basic(url.username(), password).0.encode().as_bytes().to_vec()
		});

		let headers = maybe_encoded.as_ref().map(|head| {
			[soketto::handshake::client::Header { name: "Authorization", value: head }]
		});

		if let Some(ref head) = headers {
			client.set_headers(head);
		}
		let handshake = client.handshake();
		let (sender, receiver) = match handshake.await? {
			ServerResponse::Accepted { .. } => client.into_builder().finish(),
			ServerResponse::Redirect { status_code, .. } =>
				return Err(error::Error::Transport(TransportError::Code(status_code))),
			ServerResponse::Rejected { status_code } =>
				return Err(error::Error::Transport(TransportError::Code(status_code))),
		};

		Ok(WsServerTask {
			sender,
			receiver,
			pending: Default::default(),
			subscriptions: Default::default(),
		})
	}

	async fn into_task(self, requests: mpsc::UnboundedReceiver<TransportMessage>) {
		let Self { receiver, mut sender, mut pending, mut subscriptions } = self;

		let receiver = as_data_stream(receiver).fuse();
		let requests = requests.fuse();
		pin_mut!(receiver);
		pin_mut!(requests);
		loop {
			select! {
				msg = requests.next() => match msg {
					Some(TransportMessage::Request { id, request, sender: tx }) => {
						if pending.insert(id, tx).is_some() {
							println!("Replacing a pending request with id {:?}", id);
						}
						let res = sender.send_text(request).await;
						let res2 = sender.flush().await;

						if id > 2 {
							println!("!!! Simulating connection error!!!");
							break;
						}
						if let Err(e) = res.and(res2) {
							// TODO [ToDr] Re-connect.
							println!("WS connection error: {:?}", e);

							// MAXIM: We want to keep all current subscriptions and all pending requests?
							// The other side won't keep our subscriptions if the connection is dropped
							//
							pending.remove(&id);
							// Might as well finish the entire async task. The "front end" end should be
							// able to detect this failure
							break;
						}
					}
					Some(TransportMessage::Subscribe { id, sink }) => {
						if subscriptions.insert(id.clone(), sink).is_some() {
							println!("Replacing already-registered subscription with id {:?}", id);
						}
					}
					Some(TransportMessage::Unsubscribe { id }) => {
						if subscriptions.remove(&id).is_none() {
							println!("Unsubscribing from non-existent subscription with id {:?}", id);
						}
					}
					None => {}
				},
				res = receiver.next() => match res {
					Some(Ok(data)) => {
						handle_message(&data, &subscriptions, &mut pending);
					},
					Some(Err(e)) => {
						println!("WS connection error: {:?}", e);
						break;
					},
					None => break,
				},
				complete => break,
			}
		}

		println!("Finished async task");
	}
}

fn handle_message(
	data: &[u8],
	subscriptions: &BTreeMap<SubscriptionId, Subscription>,
	pending: &mut BTreeMap<RequestId, Pending>,
) {
	println!("Message received: {:?}", data);
	if let Ok(notification) = helpers::to_notification_from_slice(data) {
		println!("Received notifications");
		if let rpc::Params::Map(params) = notification.params {
			let id = params.get("subscription");
			let result = params.get("result");

			if let (Some(&rpc::Value::String(ref id)), Some(result)) = (id, result) {
				let id: SubscriptionId = id.clone().into();
				println!("Got message for subscription id: {:?}", id);
				if let Some(stream) = subscriptions.get(&id) {
					if let Err(e) = stream.unbounded_send(result.clone()) {
						println!("Error sending notification: {:?} (id: {:?}", e, id);
					}
				} else {
					println!("Got notification for unknown subscription (id: {:?})", id);
				}
			} else {
				println!("Got unsupported notification (id: {:?})", id);
			}
		}
	} else {
		println!("Received regular response");
		let response = helpers::to_response_from_slice(data);
		let outputs = match response {
			Ok(rpc::Response::Single(output)) => vec![output],
			Ok(rpc::Response::Batch(outputs)) => outputs,
			_ => vec![],
		};

		let id = match outputs.get(0) {
			Some(&rpc::Output::Success(ref success)) => success.id.clone(),
			Some(&rpc::Output::Failure(ref failure)) => failure.id.clone(),
			None => rpc::Id::Num(0),
		};

		if let rpc::Id::Num(num) = id {
			if let Some(request) = pending.remove(&(num as usize)) {
				println!("Responding to (id: {:?}) with {:?}", num, outputs);
				if let Err(err) = request.send(helpers::to_results_from_outputs(outputs)) {
					println!("Sending a response to deallocated channel: {:?}", err);
				}
			} else {
				println!("Got response for unknown request (id: {:?})", num);
			}
		} else {
			println!("Got unsupported response (id: {:?})", id);
		}
	}
}

enum MaybeTlsStream<P, T> {
	/// Unencrypted socket stream.
	Plain(P),
	/// Encrypted socket stream.
	#[allow(dead_code)]
	Tls(T),
}

impl<P, T> AsyncRead for MaybeTlsStream<P, T>
where
	P: AsyncRead + AsyncWrite + Unpin,
	T: AsyncRead + AsyncWrite + Unpin,
{
	fn poll_read(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, std::io::Error>> {
		match self.get_mut() {
			MaybeTlsStream::Plain(ref mut s) => Pin::new(s).poll_read(cx, buf),
			MaybeTlsStream::Tls(ref mut s) => Pin::new(s).poll_read(cx, buf),
		}
	}
}

impl<P, T> AsyncWrite for MaybeTlsStream<P, T>
where
	P: AsyncRead + AsyncWrite + Unpin,
	T: AsyncRead + AsyncWrite + Unpin,
{
	fn poll_write(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, std::io::Error>> {
		match self.get_mut() {
			MaybeTlsStream::Plain(ref mut s) => Pin::new(s).poll_write(cx, buf),
			MaybeTlsStream::Tls(ref mut s) => Pin::new(s).poll_write(cx, buf),
		}
	}

	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), std::io::Error>> {
		match self.get_mut() {
			MaybeTlsStream::Plain(ref mut s) => Pin::new(s).poll_flush(cx),
			MaybeTlsStream::Tls(ref mut s) => Pin::new(s).poll_flush(cx),
		}
	}

	fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), std::io::Error>> {
		match self.get_mut() {
			MaybeTlsStream::Plain(ref mut s) => Pin::new(s).poll_close(cx),
			MaybeTlsStream::Tls(ref mut s) => Pin::new(s).poll_close(cx),
		}
	}
}

fn as_data_stream<T: Unpin + futures::AsyncRead + futures::AsyncWrite>(
	receiver: soketto::connection::Receiver<T>,
) -> impl Stream<Item = Result<Vec<u8>, soketto::connection::Error>> {
	futures::stream::unfold(receiver, |mut receiver| async move {
		let mut data = Vec::new();
		Some(match receiver.receive_data(&mut data).await {
			Ok(_) => (Ok(data), receiver),
			Err(e) => (Err(e), receiver),
		})
	})
}
