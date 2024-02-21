pub use cf_primitives::chains::Solana;
use cf_primitives::{AssetAmount, ChannelId};
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sol_prim::SlotNumber;

use crate::{address, assets, deposit_channel, FeeRefundCalculator};

use super::Chain;

mod chain_crypto;
mod tracked_data;
mod transaction;

pub mod api;
pub mod consts;

pub use sol_prim::{
	pda::{Pda as DerivedAddressBuilder, PdaError as AddressDerivationError},
	Address as SolAddress, Signature as SolSignature,
};

pub use chain_crypto::SolanaCrypto;
pub use tracked_data::SolTrackedData;
pub use transaction::SolTransaction;

impl Chain for Solana {
	const NAME: &'static str = "Solana";
	const GAS_ASSET: Self::ChainAsset = assets::sol::Asset::Sol;

	type ChainCrypto = SolanaCrypto;
	type ChainBlockNumber = SlotNumber;
	type ChainAmount = AssetAmount;
	type TransactionFee = Self::ChainAmount;
	type TrackedData = tracked_data::SolTrackedData;
	type ChainAsset = assets::sol::Asset;
	type ChainAccount = SolAddress;
	type EpochStartData = ();
	type DepositFetchId = ChannelId;
	type DepositChannelState = DepositChannelState;
	type DepositDetails = ();
	type Transaction = SolTransaction;
	type TransactionMetadata = ();
	type ReplayProtectionParams = ();
	type ReplayProtection = ();
}

#[derive(
	// XXX: Default shouldn't probably be here :S
	Default,
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	TypeInfo,
	Encode,
	Decode,
	MaxEncodedLen,
	serde::Serialize,
	serde::Deserialize,
)]
pub struct DepositChannelState {
	pub active_since_slot_number: SlotNumber,
}

impl deposit_channel::ChannelLifecycleHooks for DepositChannelState {}

impl FeeRefundCalculator<Solana> for SolTransaction {
	fn return_fee_refund(
		&self,
		fee_paid: <Solana as Chain>::TransactionFee,
	) -> <Solana as Chain>::ChainAmount {
		fee_paid
	}
}

impl TryFrom<address::ForeignChainAddress> for SolAddress {
	type Error = address::AddressError;
	fn try_from(value: address::ForeignChainAddress) -> Result<Self, Self::Error> {
		if let address::ForeignChainAddress::Sol(value) = value {
			Ok(value)
		} else {
			Err(address::AddressError::InvalidAddress)
		}
	}
}
impl From<SolAddress> for address::ForeignChainAddress {
	fn from(value: SolAddress) -> Self {
		address::ForeignChainAddress::Sol(value)
	}
}

impl address::ToHumanreadableAddress for SolAddress {
	#[cfg(feature = "std")]
	type Humanreadable = String;

	#[cfg(feature = "std")]
	fn to_humanreadable(
		&self,
		_network_environment: cf_primitives::NetworkEnvironment,
	) -> Self::Humanreadable {
		self.to_string()
	}
}
