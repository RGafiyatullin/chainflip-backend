use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

use crate::address;

use super::consts::SOLANA_PUBLIC_KEY_SIZE;

#[derive(
	Default,
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	TypeInfo,
	Encode,
	Decode,
	MaxEncodedLen,
	Serialize,
	Deserialize,
)]
pub struct SolPublicKey(#[serde(with = "::serde_bytes")] pub [u8; SOLANA_PUBLIC_KEY_SIZE]);

impl From<[u8; SOLANA_PUBLIC_KEY_SIZE]> for SolPublicKey {
	fn from(value: [u8; SOLANA_PUBLIC_KEY_SIZE]) -> Self {
		Self(value)
	}
}
impl From<SolPublicKey> for [u8; SOLANA_PUBLIC_KEY_SIZE] {
	fn from(value: SolPublicKey) -> Self {
		value.0
	}
}

impl TryFrom<address::ForeignChainAddress> for SolPublicKey {
	type Error = address::AddressError;
	fn try_from(value: address::ForeignChainAddress) -> Result<Self, Self::Error> {
		if let address::ForeignChainAddress::Sol(value) = value {
			Ok(value)
		} else {
			Err(address::AddressError::InvalidAddress)
		}
	}
}
impl From<SolPublicKey> for address::ForeignChainAddress {
	fn from(value: SolPublicKey) -> Self {
		address::ForeignChainAddress::Sol(value)
	}
}

impl core::str::FromStr for SolPublicKey {
	type Err = address::AddressError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let bytes = base58::FromBase58::from_base58(s)
			.map_err(|_| address::AddressError::InvalidAddress)?;
		Ok(Self(bytes.try_into().map_err(|_| address::AddressError::InvalidAddress)?))
	}
}

impl address::ToHumanreadableAddress for SolPublicKey {
	#[cfg(feature = "std")]
	type Humanreadable = Self;

	#[cfg(feature = "std")]
	fn to_humanreadable(
		&self,
		_network_environment: cf_primitives::NetworkEnvironment,
	) -> Self::Humanreadable {
		*self
	}
}
