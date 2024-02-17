use base64::Engine;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use serde_json::json;

use crate::{
	error::Error,
	responses::LatestBlockhash,
	types::{Commitment, JsValue, PrioritizationFeeRecord, Response},
};

#[async_trait::async_trait]
impl crate::traits::SolanaGetLatestBlockhash for HttpClient {
	async fn get_latest_blockhash(
		&self,
		commitment: Commitment,
	) -> Result<Response<LatestBlockhash>, Error> {
		self.request(
			"getLatestBlockhash",
			rpc_params![json!({
				"commitment": commitment,
			})],
		)
		.await
		.map_err(Error::transport)
	}
}

#[async_trait::async_trait]
impl crate::traits::SolanaGetFeeForMessage for HttpClient {
	async fn get_fee_for_message(
		&self,
		message: &[u8],
		commitment: Commitment,
	) -> Result<Response<Option<u64>>, Error> {
		let message_encoded = base64::engine::general_purpose::STANDARD.encode(message);

		self.request(
			"getFeeForMessage",
			rpc_params![
				message_encoded,
				json!({
					"commitment": commitment,
				})
			],
		)
		.await
		.map_err(Error::transport)
	}
}

#[async_trait::async_trait]
impl crate::traits::SolanaGetRecentPrioritizationFees for HttpClient {
	async fn get_recent_prioritization_fees<I>(
		&self,
		accounts: I,
	) -> Result<Vec<PrioritizationFeeRecord>, Error>
	where
		I: IntoIterator<Item = [u8; crate::types::ACCOUNT_ADDRESS_LEN]> + Send,
	{
		let accounts_encoded = accounts
			.into_iter()
			.map(|a| bs58::encode(a.as_ref()).into_string())
			.collect::<Vec<_>>();

		self.request("getRecentPrioritizationFees", rpc_params![accounts_encoded])
			.await
			.map_err(Error::transport)
	}
}

#[async_trait::async_trait]
impl crate::traits::SolanaGetGenesisHash for HttpClient {
	async fn get_genesis_hash(&self) -> Result<String, Error> {
		self.request("getGenesisHash", rpc_params![]).await.map_err(Error::transport)
	}
}

#[async_trait::async_trait]
impl crate::traits::SolanaGetTransaction for HttpClient {
	async fn get_transaction(
		&self,
		signature: &[u8; crate::types::SIGNATURE_LEN],
		commitment: Commitment,
	) -> Result<JsValue, Error> {
		let signature = bs58::encode(signature).into_string();
		self.request(
			"getTransaction",
			rpc_params![
				signature.as_str(),
				json!({
					"commitment": commitment,
				})
			],
		)
		.await
		.map_err(Error::transport)
	}
}

#[async_trait::async_trait]
impl crate::traits::SolanaGetSignaturesForTransaction for HttpClient {
	async fn get_signatures_for_transaction(
		&self,
		account: &[u8; crate::types::ACCOUNT_ADDRESS_LEN],
		commitment: Commitment,
		limit: usize,
		before: Option<&[u8; crate::types::SIGNATURE_LEN]>,
		until: Option<&[u8; crate::types::SIGNATURE_LEN]>,
	) -> Result<Vec<JsValue>, Error> {
		let account = bs58::encode(account).into_string();
		let before = before.map(|s| bs58::encode(s).into_string());
		let until = until.map(|s| bs58::encode(s).into_string());
		self.request(
			"getSignaturesForTransaction",
			rpc_params![
				account.as_str(),
				json!({
					"commitment": commitment,
					"limit": limit,
					"before": before.as_ref(),
					"until": until.as_ref(),
				})
			],
		)
		.await
		.map_err(Error::transport)
	}
}
