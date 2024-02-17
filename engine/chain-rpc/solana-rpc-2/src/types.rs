use std::collections::HashMap;

#[macro_use]
mod binary;

pub type JsValue = serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
	pub slot: u64,
	pub api_version: String,

	#[serde(flatten)]
	pub extra: HashMap<String, JsValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WithContext<Value> {
	pub context: Context,
	pub value: Value,
}

define_binary!(Hash, crate::consts::HASH_LEN);
define_binary!(Signature, crate::consts::SIGNATURE_LEN);
define_binary!(Address, crate::consts::ADDRESS_LEN);

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Commitment {
	Processed = 1,
	Confirmed = 2,
	Finalized = 3,
}

impl Default for Commitment {
	fn default() -> Self {
		Self::Finalized
	}
}
