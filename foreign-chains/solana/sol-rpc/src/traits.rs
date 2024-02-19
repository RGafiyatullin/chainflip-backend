use std::error::Error as StdError;

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

#[async_trait::async_trait]
pub trait CallApi {
	type Error: StdError + Send + Sync + 'static;
	async fn call<C: Call>(&self, call: C) -> Result<C::Response, Self::Error>;
}

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
