
//! Autogenerated weights for pallet_cf_funding
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-17, STEPS: `20`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `kylezs.localdomain`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/chainflip-node
// benchmark
// pallet
// --extrinsic
// *
// --pallet
// pallet_cf_funding
// --output
// state-chain/pallets/cf-funding/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=20
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_funding.
pub trait WeightInfo {
	fn funded() -> Weight;
	fn redeem() -> Weight;
	fn redeem_all() -> Weight;
	fn redeemed() -> Weight;
	fn redemption_expired() -> Weight;
	fn stop_bidding() -> Weight;
	fn start_bidding() -> Weight;
	fn update_minimum_funding() -> Weight;
	fn update_redemption_tax() -> Weight;
}

/// Weights for pallet_cf_funding using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Flip OffchainFunds (r:1 w:1)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator Backups (r:1 w:1)
	// Storage: Funding WithdrawalAddresses (r:0 w:1)
	// Storage: Funding ActiveBidder (r:0 w:1)
	// Storage: AccountRoles AccountRoles (r:0 w:1)
	fn funded() -> Weight {
		// Minimum execution time: 51_000 nanoseconds.
		Weight::from_ref_time(53_000_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator CurrentEpochStartedAt (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:0)
	// Storage: Validator RedemptionPeriodAsPercentage (r:1 w:0)
	// Storage: Funding PendingRedemptions (r:1 w:1)
	// Storage: Funding WithdrawalAddresses (r:1 w:0)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Funding MinimumFunding (r:1 w:0)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator Backups (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Funding RedemptionTTLSeconds (r:1 w:0)
	// Storage: Environment EthereumKeyManagerAddress (r:1 w:0)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: Environment EthereumSignatureNonce (r:1 w:1)
	// Storage: EthereumBroadcaster BroadcastIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureRequestIdCounter (r:1 w:1)
	// Storage: EthereumVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: Validator EpochAuthorityCount (r:1 w:0)
	// Storage: Reputation Suspensions (r:3 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Validator CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:0)
	// Storage: EthereumThresholdSigner CeremonyRetryQueues (r:1 w:1)
	// Storage: EthereumThresholdSigner Signature (r:0 w:1)
	// Storage: EthereumThresholdSigner PendingCeremonies (r:0 w:1)
	// Storage: EthereumThresholdSigner RequestCallback (r:0 w:1)
	// Storage: Flip PendingRedemptionsReserve (r:0 w:1)
	fn redeem() -> Weight {
		// Minimum execution time: 130_000 nanoseconds.
		Weight::from_ref_time(132_000_000)
			.saturating_add(T::DbWeight::get().reads(28))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator CurrentEpochStartedAt (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:0)
	// Storage: Validator RedemptionPeriodAsPercentage (r:1 w:0)
	// Storage: Funding PendingRedemptions (r:1 w:1)
	// Storage: Funding WithdrawalAddresses (r:1 w:0)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator Backups (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Funding RedemptionTTLSeconds (r:1 w:0)
	// Storage: Environment EthereumKeyManagerAddress (r:1 w:0)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: Environment EthereumSignatureNonce (r:1 w:1)
	// Storage: EthereumBroadcaster BroadcastIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureRequestIdCounter (r:1 w:1)
	// Storage: EthereumVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: Validator EpochAuthorityCount (r:1 w:0)
	// Storage: Reputation Suspensions (r:3 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Validator CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:0)
	// Storage: EthereumThresholdSigner CeremonyRetryQueues (r:1 w:1)
	// Storage: EthereumThresholdSigner Signature (r:0 w:1)
	// Storage: EthereumThresholdSigner PendingCeremonies (r:0 w:1)
	// Storage: EthereumThresholdSigner RequestCallback (r:0 w:1)
	// Storage: Flip PendingRedemptionsReserve (r:0 w:1)
	fn redeem_all() -> Weight {
		// Minimum execution time: 130_000 nanoseconds.
		Weight::from_ref_time(131_000_000)
			.saturating_add(T::DbWeight::get().reads(27))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	// Storage: Funding PendingRedemptions (r:1 w:1)
	// Storage: Flip PendingRedemptionsReserve (r:1 w:1)
	// Storage: Flip OffchainFunds (r:1 w:1)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Flip TotalIssuance (r:1 w:1)
	// Storage: Validator AccountPeerMapping (r:1 w:0)
	// Storage: Validator VanityNames (r:1 w:1)
	// Storage: Reputation LastHeartbeat (r:0 w:1)
	// Storage: Reputation Reputations (r:0 w:1)
	// Storage: Reputation OffenceTimeSlotTracker (r:0 w:1)
	// Storage: Funding WithdrawalAddresses (r:0 w:1)
	// Storage: Funding ActiveBidder (r:0 w:1)
	// Storage: AccountRoles AccountRoles (r:0 w:1)
	fn redeemed() -> Weight {
		// Minimum execution time: 73_000 nanoseconds.
		Weight::from_ref_time(75_000_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	// Storage: Funding PendingRedemptions (r:1 w:1)
	// Storage: Flip PendingRedemptionsReserve (r:1 w:1)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator Backups (r:1 w:1)
	fn redemption_expired() -> Weight {
		// Minimum execution time: 42_000 nanoseconds.
		Weight::from_ref_time(44_000_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator CurrentEpochStartedAt (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:0)
	// Storage: Validator RedemptionPeriodAsPercentage (r:1 w:0)
	// Storage: Funding ActiveBidder (r:1 w:1)
	fn stop_bidding() -> Weight {
		// Minimum execution time: 36_000 nanoseconds.
		Weight::from_ref_time(37_000_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Funding ActiveBidder (r:1 w:1)
	fn start_bidding() -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_ref_time(28_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Funding MinimumFunding (r:0 w:1)
	fn update_minimum_funding() -> Weight {
		// Minimum execution time: 14_000 nanoseconds.
		Weight::from_ref_time(14_000_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}

	fn update_redemption_tax() -> Weight{
		Weight::from_ref_time(1_000_000)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Flip OffchainFunds (r:1 w:1)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator Backups (r:1 w:1)
	// Storage: Funding WithdrawalAddresses (r:0 w:1)
	// Storage: Funding ActiveBidder (r:0 w:1)
	// Storage: AccountRoles AccountRoles (r:0 w:1)
	fn funded() -> Weight {
		// Minimum execution time: 51_000 nanoseconds.
		Weight::from_ref_time(53_000_000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator CurrentEpochStartedAt (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:0)
	// Storage: Validator RedemptionPeriodAsPercentage (r:1 w:0)
	// Storage: Funding PendingRedemptions (r:1 w:1)
	// Storage: Funding WithdrawalAddresses (r:1 w:0)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Funding MinimumFunding (r:1 w:0)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator Backups (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Funding RedemptionTTLSeconds (r:1 w:0)
	// Storage: Environment EthereumKeyManagerAddress (r:1 w:0)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: Environment EthereumSignatureNonce (r:1 w:1)
	// Storage: EthereumBroadcaster BroadcastIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureRequestIdCounter (r:1 w:1)
	// Storage: EthereumVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: Validator EpochAuthorityCount (r:1 w:0)
	// Storage: Reputation Suspensions (r:3 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Validator CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:0)
	// Storage: EthereumThresholdSigner CeremonyRetryQueues (r:1 w:1)
	// Storage: EthereumThresholdSigner Signature (r:0 w:1)
	// Storage: EthereumThresholdSigner PendingCeremonies (r:0 w:1)
	// Storage: EthereumThresholdSigner RequestCallback (r:0 w:1)
	// Storage: Flip PendingRedemptionsReserve (r:0 w:1)
	fn redeem() -> Weight {
		// Minimum execution time: 130_000 nanoseconds.
		Weight::from_ref_time(132_000_000)
			.saturating_add(RocksDbWeight::get().reads(28))
			.saturating_add(RocksDbWeight::get().writes(12))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator CurrentEpochStartedAt (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:0)
	// Storage: Validator RedemptionPeriodAsPercentage (r:1 w:0)
	// Storage: Funding PendingRedemptions (r:1 w:1)
	// Storage: Funding WithdrawalAddresses (r:1 w:0)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator Backups (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Funding RedemptionTTLSeconds (r:1 w:0)
	// Storage: Environment EthereumKeyManagerAddress (r:1 w:0)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: Environment EthereumSignatureNonce (r:1 w:1)
	// Storage: EthereumBroadcaster BroadcastIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureRequestIdCounter (r:1 w:1)
	// Storage: EthereumVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: Validator EpochAuthorityCount (r:1 w:0)
	// Storage: Reputation Suspensions (r:3 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Validator CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:0)
	// Storage: EthereumThresholdSigner CeremonyRetryQueues (r:1 w:1)
	// Storage: EthereumThresholdSigner Signature (r:0 w:1)
	// Storage: EthereumThresholdSigner PendingCeremonies (r:0 w:1)
	// Storage: EthereumThresholdSigner RequestCallback (r:0 w:1)
	// Storage: Flip PendingRedemptionsReserve (r:0 w:1)
	fn redeem_all() -> Weight {
		// Minimum execution time: 130_000 nanoseconds.
		Weight::from_ref_time(131_000_000)
			.saturating_add(RocksDbWeight::get().reads(27))
			.saturating_add(RocksDbWeight::get().writes(12))
	}
	// Storage: Funding PendingRedemptions (r:1 w:1)
	// Storage: Flip PendingRedemptionsReserve (r:1 w:1)
	// Storage: Flip OffchainFunds (r:1 w:1)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Flip TotalIssuance (r:1 w:1)
	// Storage: Validator AccountPeerMapping (r:1 w:0)
	// Storage: Validator VanityNames (r:1 w:1)
	// Storage: Reputation LastHeartbeat (r:0 w:1)
	// Storage: Reputation Reputations (r:0 w:1)
	// Storage: Reputation OffenceTimeSlotTracker (r:0 w:1)
	// Storage: Funding WithdrawalAddresses (r:0 w:1)
	// Storage: Funding ActiveBidder (r:0 w:1)
	// Storage: AccountRoles AccountRoles (r:0 w:1)
	fn redeemed() -> Weight {
		// Minimum execution time: 73_000 nanoseconds.
		Weight::from_ref_time(75_000_000)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(12))
	}
	// Storage: Funding PendingRedemptions (r:1 w:1)
	// Storage: Flip PendingRedemptionsReserve (r:1 w:1)
	// Storage: Flip Account (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator Backups (r:1 w:1)
	fn redemption_expired() -> Weight {
		// Minimum execution time: 42_000 nanoseconds.
		Weight::from_ref_time(44_000_000)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator CurrentEpochStartedAt (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:0)
	// Storage: Validator RedemptionPeriodAsPercentage (r:1 w:0)
	// Storage: Funding ActiveBidder (r:1 w:1)
	fn stop_bidding() -> Weight {
		// Minimum execution time: 36_000 nanoseconds.
		Weight::from_ref_time(37_000_000)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Funding ActiveBidder (r:1 w:1)
	fn start_bidding() -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_ref_time(28_000_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Funding MinimumFunding (r:0 w:1)
	fn update_minimum_funding() -> Weight {
		// Minimum execution time: 14_000 nanoseconds.
		Weight::from_ref_time(14_000_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}

	fn update_redemption_tax() -> Weight{
		Weight::from_ref_time(1_000_000)
	}
}
