use crate::{
	error::Error,
	responses::LatestBlockhash,
	types::{Commitment, JsValue, PrioritizationFeeRecord, Response},
};

#[async_trait::async_trait]
pub trait SolanaGetLatestBlockhash {
	async fn get_latest_blockhash(
		&self,
		commitment: Commitment,
	) -> Result<Response<LatestBlockhash>, Error>;
}

#[async_trait::async_trait]
pub trait SolanaGetFeeForMessage {
	async fn get_fee_for_message(
		&self,
		message: &[u8],
		commitment: Commitment,
	) -> Result<Response<Option<u64>>, Error>;
}

#[async_trait::async_trait]
pub trait SolanaGetRecentPrioritizationFees {
	async fn get_recent_prioritization_fees<I>(
		&self,
		accounts: I,
	) -> Result<Vec<PrioritizationFeeRecord>, Error>
	where
		I: IntoIterator<Item = [u8; crate::types::ACCOUNT_ADDRESS_LEN]> + Send;
}

#[async_trait::async_trait]
pub trait SolanaGetGenesisHash {
	async fn get_genesis_hash(&self) -> Result<String, Error>;
}

#[async_trait::async_trait]
pub trait SolanaGetTransaction {
	async fn get_transaction(
		&self,
		signature: &[u8; crate::types::SIGNATURE_LEN],
		commitment: Commitment,
	) -> Result<JsValue, Error>;
}

#[async_trait::async_trait]
pub trait SolanaGetSignaturesForTransaction {
	async fn get_signatures_for_transaction(
		&self,
		account: &[u8; crate::types::ACCOUNT_ADDRESS_LEN],
		commitment: Commitment,
		limit: usize,
		before: Option<&[u8; crate::types::SIGNATURE_LEN]>,
		until: Option<&[u8; crate::types::SIGNATURE_LEN]>,
	) -> Result<Vec<JsValue>, Error>;
}

blanket_impl!(
	SolanaCallApi,
	[
		SolanaGetGenesisHash,
		SolanaGetLatestBlockhash,
		SolanaGetFeeForMessage,
		SolanaGetTransaction,
		SolanaGetRecentPrioritizationFees,
		SolanaGetSignaturesForTransaction
	]
);
blanket_impl!(SolanaApi, [SolanaCallApi]);
