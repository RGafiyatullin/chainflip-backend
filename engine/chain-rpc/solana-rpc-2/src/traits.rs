use jsonrpsee::core::params::ArrayParams;

use crate::types::JsValue;

pub trait Call: Send + Sync {
	type Response: serde::de::DeserializeOwned + Send;

	const CALL_METHOD_NAME: &'static str;
	fn call_params(&self) -> ArrayParams;

	fn process_response(&self, input: JsValue) -> Result<Self::Response, serde_json::Error> {
		serde_json::from_value(input)
	}
}

// pub trait Subscription: Send + Sync {
// 	type Notification;

// 	const SUBSCRIBE_METHOD_NAME: &'static str;
// 	const UNSUBSCRIBE_METHOD_NAME: &'static str;
// 	fn subscribe_params(&self) -> ArrayParams;
// }

impl<'a, C> Call for &'a C
where
	C: Call,
{
	type Response = C::Response;

	const CALL_METHOD_NAME: &'static str = C::CALL_METHOD_NAME;
	fn call_params(&self) -> ArrayParams {
		<C as Call>::call_params(*self)
	}
}

// impl<'a, S> Subscription for &'a S
// where
// 	S: Subscription,
// {
// 	type Notification = S::Notification;
// 	const SUBSCRIBE_METHOD_NAME: &'static str = S::SUBSCRIBE_METHOD_NAME;
// 	const UNSUBSCRIBE_METHOD_NAME: &'static str = S::UNSUBSCRIBE_METHOD_NAME;
// 	fn subscribe_params(&self) -> ArrayParams {
// 		<S as Subscription>::subscribe_params(*self)
// 	}
// }

#[async_trait::async_trait]
pub trait CallApi {
	type Error: Send;
	async fn call<C: Call>(&self, call: C) -> Result<C::Response, Self::Error>;
}
