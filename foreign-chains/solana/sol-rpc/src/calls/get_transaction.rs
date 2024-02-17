use std::collections::HashMap;

use jsonrpsee::rpc_params;
use serde_json::json;

use sol_prim::{address::Address, signature::Signature};

use super::GetTransaction;
use crate::{traits::Call, types::JsValue};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxMessage {
	pub account_keys: Vec<Address>,
	pub header: HashMap<String, JsValue>,
	pub instructions: Vec<JsValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInfo {
	pub message: TxMessage,
	pub signatures: Vec<Signature>,

	#[serde(flatten)]
	extra: HashMap<String, JsValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedAddresses {
	pub readonly: Vec<Address>,
	pub writable: Vec<Address>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxMeta {
	pub log_messages: Vec<String>,
	pub err: Option<JsValue>,
	pub pre_balances: Vec<u64>,
	pub post_balances: Vec<u64>,
	pub fee: u64,
	pub loaded_addresses: LoadedAddresses,

	#[serde(flatten)]
	extra: HashMap<String, JsValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
	pub slot: u64,
	pub block_time: u64,
	pub transaction: TxInfo,
	pub meta: TxMeta,
}

impl Call for GetTransaction {
	type Response = Response;
	const CALL_METHOD_NAME: &'static str = "getTransaction";
	fn call_params(&self) -> jsonrpsee::core::params::ArrayParams {
		let signature = self.signature.to_string();
		rpc_params![
			signature.as_str(),
			json!({
				"commitment": self.commitment,
			})
		]
	}
}

impl GetTransaction {
	pub fn for_signature(signature: Signature) -> Self {
		Self { signature, commitment: Default::default() }
	}
}

impl Response {
	pub fn addresses(&self) -> impl Iterator<Item = &Address> + '_ {
		self.transaction.message.account_keys.iter()
	}

	pub fn balances(&self, address: &Address) -> Option<(u64, u64)> {
		let account_idx =
			self.transaction.message.account_keys.iter().position(|a| a == address)?;
		Some((
			self.meta.pre_balances.get(account_idx).copied()?,
			self.meta.post_balances.get(account_idx).copied()?,
		))
	}
}
