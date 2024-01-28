use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::RuntimeDebug;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

use cf_primitives::chains::assets;

use crate::{deposit_channel, Chain, FeeEstimationApi, FeeRefundCalculator};

use super::{Solana, SolanaCrypto};

pub type BlockNumber = u64;
pub type SolAmount = u64;

#[derive(
	Copy,
	Clone,
	RuntimeDebug,
	PartialEq,
	Eq,
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
	Serialize,
	Deserialize,
)]
pub struct Stub;

#[derive(
	Copy,
	Clone,
	RuntimeDebug,
	PartialEq,
	Eq,
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
	Serialize,
	Deserialize,
	Default,
)]
pub struct SolanaTrackedData;

#[derive(
	Copy,
	Clone,
	RuntimeDebug,
	PartialEq,
	Eq,
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
	Serialize,
	Deserialize,
	Default,
)]
pub struct DepositFetchId;

impl Chain for Solana {
	const NAME: &'static str = "Solana";
	const GAS_ASSET: Self::ChainAsset = assets::sol::Asset::Sol;

	type ChainCrypto = SolanaCrypto;

	type ChainBlockNumber = BlockNumber;
	type ChainAmount = SolAmount;
	type TransactionFee = SolAmount;
	type TrackedData = SolanaTrackedData;
	type ChainAccount = super::Pubkey;
	type ChainAsset = assets::sol::Asset;
	type EpochStartData = ();
	type DepositFetchId = DepositFetchId;
	type DepositChannelState = ();
	type DepositDetails = ();
	type Transaction = super::Transaction;
	type TransactionMetadata = ();
	type ReplayProtectionParams = ();
	type ReplayProtection = ();
}

impl FeeEstimationApi<Solana> for SolanaTrackedData {
	fn estimate_ingress_fee(
		&self,
		_asset: <Solana as Chain>::ChainAsset,
	) -> <Solana as Chain>::ChainAmount {
		unimplemented!()
	}
	fn estimate_egress_fee(
		&self,
		_asset: <Solana as Chain>::ChainAsset,
	) -> <Solana as Chain>::ChainAmount {
		unimplemented!()
	}
}

impl<'a> From<&'a deposit_channel::DepositChannel<Solana>> for DepositFetchId {
	fn from(_value: &'a deposit_channel::DepositChannel<Solana>) -> Self {
		unimplemented!()
	}
}

impl FeeRefundCalculator<Solana> for super::Transaction {
	fn return_fee_refund(
		&self,
		_fee_paid: <Solana as Chain>::TransactionFee,
	) -> <Solana as Chain>::ChainAmount {
		unimplemented!()
	}
}
