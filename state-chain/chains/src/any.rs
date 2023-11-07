use crate::{
	address::ForeignChainAddress, none::NoneChainCrypto, Chain, FeeRefundCalculator,
	NoDepositTracking,
};

use cf_primitives::{
	chains::{assets, AnyChain},
	AssetAmount,
};

impl Chain for AnyChain {
	const NAME: &'static str = "AnyChain";
	type ChainCrypto = NoneChainCrypto;
	type ChainBlockNumber = u64;
	type ChainAmount = AssetAmount;
	type TransactionFee = Self::ChainAmount;
	type TrackedData = ();
	type ChainAsset = assets::any::Asset;
	type ChainAccount = ForeignChainAddress;
	type EpochStartData = ();
	type FetchParams = ();
	type DepositDetails = ();
	type DepositChannel = ();
	type DepositTracker = NoDepositTracking<Self>;
	type Transaction = ();
	type TransactionMetadata = ();
	type ReplayProtectionParams = ();
	type ReplayProtection = ();
}

impl FeeRefundCalculator<AnyChain> for () {
	fn return_fee_refund(
		&self,
		_fee_paid: <AnyChain as Chain>::TransactionFee,
	) -> <AnyChain as Chain>::ChainAmount {
		unimplemented!()
	}
}
