use jsonrpsee::rpc_params;
use serde_json::json;

use super::GetLatestBlockhash;
use crate::{traits::Call, types::WithContext};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
	#[serde(with = "crate::utils::serde_b58")]
	pub blockhash: [u8; crate::consts::HASH_LEN],

	pub last_valid_block_height: u64,
}

impl Call for GetLatestBlockhash {
	type Response = WithContext<Response>;
	const CALL_METHOD_NAME: &'static str = "getLatestBlockhash";

	fn call_params(&self) -> jsonrpsee::core::params::ArrayParams {
		rpc_params![json!({
			"commitment": self.commitment,
		})]
	}
}
