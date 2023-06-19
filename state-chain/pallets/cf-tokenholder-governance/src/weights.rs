
//! Autogenerated weights for pallet_cf_tokenholder_governance
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-19, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ip-172-31-12-158`, CPU: `Intel(R) Xeon(R) Platinum 8275CL CPU @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// /usr/bin/chainflip-node
// benchmark
// pallet
// --pallet
// pallet_cf_tokenholder_governance
// --extrinsic
// *
// --output
// state-chain/pallets/cf-tokenholder-governance/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_tokenholder_governance.
pub trait WeightInfo {
	fn on_initialize_resolve_votes(a: u32, ) -> Weight;
	fn on_initialize_execute_proposal() -> Weight;
	fn submit_proposal() -> Weight;
	fn back_proposal(a: u32, ) -> Weight;
}

/// Weights for pallet_cf_tokenholder_governance using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: TokenholderGovernance Proposals (r:1 w:1)
	// Storage: TokenholderGovernance Backers (r:1 w:1)
	// Storage: Flip Account (r:10 w:0)
	// Storage: Flip TotalIssuance (r:1 w:0)
	// Storage: Flip OffchainFunds (r:1 w:0)
	// Storage: TokenholderGovernance CommKeyUpdateAwaitingEnactment (r:1 w:0)
	// Storage: TokenholderGovernance GovKeyUpdateAwaitingEnactment (r:0 w:1)
	/// The range of component `a` is `[10, 1000]`.
	fn on_initialize_resolve_votes(a: u32, ) -> Weight {
		// Minimum execution time: 96_732 nanoseconds.
		Weight::from_ref_time(5_221_389)
			// Standard Error: 7_259
			.saturating_add(Weight::from_ref_time(4_693_311).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: TokenholderGovernance Proposals (r:1 w:0)
	// Storage: TokenholderGovernance GovKeyUpdateAwaitingEnactment (r:1 w:1)
	// Storage: TokenholderGovernance GovKeys (r:1 w:1)
	// Storage: Environment EthereumSignatureNonce (r:1 w:1)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: Environment EthereumKeyManagerAddress (r:1 w:0)
	// Storage: EthereumBroadcaster BroadcastIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureRequestIdCounter (r:1 w:1)
	// Storage: EthereumVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Reputation Suspensions (r:3 w:0)
	// Storage: EthereumVault CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:0)
	// Storage: EthereumThresholdSigner CeremonyRetryQueues (r:1 w:1)
	// Storage: TokenholderGovernance CommKeyUpdateAwaitingEnactment (r:1 w:0)
	// Storage: EthereumThresholdSigner Signature (r:0 w:1)
	// Storage: EthereumThresholdSigner PendingCeremonies (r:0 w:1)
	// Storage: EthereumThresholdSigner RequestCallback (r:0 w:1)
	// Storage: EthereumBroadcaster RotationBroadcast (r:0 w:1)
	fn on_initialize_execute_proposal() -> Weight {
		// Minimum execution time: 143_817 nanoseconds.
		Weight::from_ref_time(146_284_000)
			.saturating_add(T::DbWeight::get().reads(18))
			.saturating_add(T::DbWeight::get().writes(11))
	}
	// Storage: Flip Account (r:1 w:1)
	// Storage: Flip TotalIssuance (r:1 w:1)
	// Storage: TokenholderGovernance Backers (r:0 w:1)
	// Storage: TokenholderGovernance Proposals (r:0 w:1)
	fn submit_proposal() -> Weight {
		// Minimum execution time: 49_351 nanoseconds.
		Weight::from_ref_time(49_983_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: TokenholderGovernance Backers (r:1 w:1)
	/// The range of component `a` is `[1, 1000]`.
	fn back_proposal(a: u32, ) -> Weight {
		// Minimum execution time: 19_348 nanoseconds.
		Weight::from_ref_time(22_632_461)
			// Standard Error: 496
			.saturating_add(Weight::from_ref_time(70_302).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: TokenholderGovernance Proposals (r:1 w:1)
	// Storage: TokenholderGovernance Backers (r:1 w:1)
	// Storage: Flip Account (r:10 w:0)
	// Storage: Flip TotalIssuance (r:1 w:0)
	// Storage: Flip OffchainFunds (r:1 w:0)
	// Storage: TokenholderGovernance CommKeyUpdateAwaitingEnactment (r:1 w:0)
	// Storage: TokenholderGovernance GovKeyUpdateAwaitingEnactment (r:0 w:1)
	/// The range of component `a` is `[10, 1000]`.
	fn on_initialize_resolve_votes(a: u32, ) -> Weight {
		// Minimum execution time: 96_732 nanoseconds.
		Weight::from_ref_time(5_221_389)
			// Standard Error: 7_259
			.saturating_add(Weight::from_ref_time(4_693_311).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: TokenholderGovernance Proposals (r:1 w:0)
	// Storage: TokenholderGovernance GovKeyUpdateAwaitingEnactment (r:1 w:1)
	// Storage: TokenholderGovernance GovKeys (r:1 w:1)
	// Storage: Environment EthereumSignatureNonce (r:1 w:1)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: Environment EthereumKeyManagerAddress (r:1 w:0)
	// Storage: EthereumBroadcaster BroadcastIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureRequestIdCounter (r:1 w:1)
	// Storage: EthereumVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Reputation Suspensions (r:3 w:0)
	// Storage: EthereumVault CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:0)
	// Storage: EthereumThresholdSigner CeremonyRetryQueues (r:1 w:1)
	// Storage: TokenholderGovernance CommKeyUpdateAwaitingEnactment (r:1 w:0)
	// Storage: EthereumThresholdSigner Signature (r:0 w:1)
	// Storage: EthereumThresholdSigner PendingCeremonies (r:0 w:1)
	// Storage: EthereumThresholdSigner RequestCallback (r:0 w:1)
	// Storage: EthereumBroadcaster RotationBroadcast (r:0 w:1)
	fn on_initialize_execute_proposal() -> Weight {
		// Minimum execution time: 143_817 nanoseconds.
		Weight::from_ref_time(146_284_000)
			.saturating_add(RocksDbWeight::get().reads(18))
			.saturating_add(RocksDbWeight::get().writes(11))
	}
	// Storage: Flip Account (r:1 w:1)
	// Storage: Flip TotalIssuance (r:1 w:1)
	// Storage: TokenholderGovernance Backers (r:0 w:1)
	// Storage: TokenholderGovernance Proposals (r:0 w:1)
	fn submit_proposal() -> Weight {
		// Minimum execution time: 49_351 nanoseconds.
		Weight::from_ref_time(49_983_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: TokenholderGovernance Backers (r:1 w:1)
	/// The range of component `a` is `[1, 1000]`.
	fn back_proposal(a: u32, ) -> Weight {
		// Minimum execution time: 19_348 nanoseconds.
		Weight::from_ref_time(22_632_461)
			// Standard Error: 496
			.saturating_add(Weight::from_ref_time(70_302).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
