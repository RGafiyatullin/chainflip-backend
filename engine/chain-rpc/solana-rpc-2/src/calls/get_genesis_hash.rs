use jsonrpsee::rpc_params;

use super::GetGenesisHash;
use crate::traits::Call;

impl Call for GetGenesisHash {
	type Response = crate::types::Hash;
	const CALL_METHOD_NAME: &'static str = "getGenesisHash";
	fn call_params(&self) -> jsonrpsee::core::params::ArrayParams {
		rpc_params![]
	}
}
