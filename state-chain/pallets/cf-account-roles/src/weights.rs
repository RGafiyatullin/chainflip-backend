
//! Autogenerated weights for pallet_cf_account_roles
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-24, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `kylezs.localdomain`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/chainflip-node
// benchmark
// pallet
// --extrinsic
// *
// --pallet
// pallet_cf_account-roles
// --output
// state-chain/pallets/cf-account-roles/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_account_roles.
pub trait WeightInfo {
	fn register_account_role() -> Weight;
	fn enable_swapping() -> Weight;
	fn gov_register_account_role() -> Weight;
}

/// Weights for pallet_cf_account_roles using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Flip Account (r:1 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_account_role() -> Weight {
		// Minimum execution time: 30_000 nanoseconds.
		Weight::from_ref_time(31_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles SwappingEnabled (r:0 w:1)
	fn enable_swapping() -> Weight {
		// Minimum execution time: 5_000 nanoseconds.
		Weight::from_ref_time(5_000_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn gov_register_account_role() -> Weight {
		// Minimum execution time: 19_000 nanoseconds.
		Weight::from_ref_time(19_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Flip Account (r:1 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_account_role() -> Weight {
		// Minimum execution time: 30_000 nanoseconds.
		Weight::from_ref_time(31_000_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles SwappingEnabled (r:0 w:1)
	fn enable_swapping() -> Weight {
		// Minimum execution time: 5_000 nanoseconds.
		Weight::from_ref_time(5_000_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn gov_register_account_role() -> Weight {
		// Minimum execution time: 19_000 nanoseconds.
		Weight::from_ref_time(19_000_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
