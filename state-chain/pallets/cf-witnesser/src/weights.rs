
//! Autogenerated weights for pallet_cf_witnesser
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
// pallet_cf_witnesser
// --extrinsic
// *
// --output
// state-chain/pallets/cf-witnesser/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_witnesser.
pub trait WeightInfo {
	fn witness_at_epoch() -> Weight;
	fn remove_storage_items(n: u32, ) -> Weight;
	fn on_idle_with_nothing_to_remove() -> Weight;
}

/// Weights for pallet_cf_witnesser using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator LastExpiredEpoch (r:1 w:0)
	// Storage: Validator CurrentEpoch (r:1 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Validator AuthorityIndex (r:1 w:0)
	// Storage: Witnesser Votes (r:1 w:1)
	// Storage: Witnesser CallHashExecuted (r:2 w:1)
	// Storage: Witnesser ExtraCallData (r:1 w:0)
	fn witness_at_epoch() -> Weight {
		// Minimum execution time: 80_719 nanoseconds.
		Weight::from_ref_time(82_438_000)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Witnesser Votes (r:0 w:1)
	/// The range of component `n` is `[1, 255]`.
	fn remove_storage_items(n: u32, ) -> Weight {
		// Minimum execution time: 6_833 nanoseconds.
		Weight::from_ref_time(5_923_942)
			// Standard Error: 4_560
			.saturating_add(Weight::from_ref_time(1_242_881).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
	}
	// Storage: Witnesser EpochsToCull (r:1 w:0)
	fn on_idle_with_nothing_to_remove() -> Weight {
		// Minimum execution time: 6_994 nanoseconds.
		Weight::from_ref_time(7_206_000)
			.saturating_add(T::DbWeight::get().reads(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator LastExpiredEpoch (r:1 w:0)
	// Storage: Validator CurrentEpoch (r:1 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Validator AuthorityIndex (r:1 w:0)
	// Storage: Witnesser Votes (r:1 w:1)
	// Storage: Witnesser CallHashExecuted (r:2 w:1)
	// Storage: Witnesser ExtraCallData (r:1 w:0)
	fn witness_at_epoch() -> Weight {
		// Minimum execution time: 80_719 nanoseconds.
		Weight::from_ref_time(82_438_000)
			.saturating_add(RocksDbWeight::get().reads(9))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Witnesser Votes (r:0 w:1)
	/// The range of component `n` is `[1, 255]`.
	fn remove_storage_items(n: u32, ) -> Weight {
		// Minimum execution time: 6_833 nanoseconds.
		Weight::from_ref_time(5_923_942)
			// Standard Error: 4_560
			.saturating_add(Weight::from_ref_time(1_242_881).saturating_mul(n.into()))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(n.into())))
	}
	// Storage: Witnesser EpochsToCull (r:1 w:0)
	fn on_idle_with_nothing_to_remove() -> Weight {
		// Minimum execution time: 6_994 nanoseconds.
		Weight::from_ref_time(7_206_000)
			.saturating_add(RocksDbWeight::get().reads(1))
	}
}
