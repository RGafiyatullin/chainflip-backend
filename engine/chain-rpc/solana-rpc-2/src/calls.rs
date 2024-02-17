use crate::types::{Commitment, Signature};

mod get_genesis_hash;
mod get_latest_blockhash;
mod get_transaction;

#[derive(Debug, Clone, Default)]
pub struct GetGenesisHash {}

#[derive(Debug, Clone, Default)]
pub struct GetLatestBlockhash {
	pub commitment: Commitment,
}

#[derive(Debug, Clone)]
pub struct GetTransaction {
	pub signature: Signature,
	pub commitment: Commitment,
}
