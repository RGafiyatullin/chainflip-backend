use sol_prim::{Address, Signature};

use crate::types::Commitment;

mod get_genesis_hash;
mod get_latest_blockhash;
mod get_signatures_for_address;
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

#[derive(Debug, Clone)]
pub struct GetSignaturesForAddress {
	pub address: Address,
	pub before: Option<Signature>,
	pub until: Option<Signature>,
	pub commitment: Commitment,
	pub limit: Option<usize>,
	pub min_context_slot: Option<u64>,
}
