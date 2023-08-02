
//! Autogenerated weights for pallet_cf_flip
//!
//! THIS FILE WAS AUTO-GENERATED USING THE CHAINFLIP BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-20, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `wagmi.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/production/chainflip-node
// benchmark
// pallet
// --pallet
// pallet_cf_flip
// --extrinsic
// *
// --output
// state-chain/pallets/cf-flip/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_flip.
pub trait WeightInfo {
	fn set_slashing_rate() -> Weight;
	fn reap_one_account() -> Weight;
}

/// Weights for pallet_cf_flip using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Flip SlashingRate (r:0 w:1)
	fn set_slashing_rate() -> Weight {
		// Minimum execution time: 11_000 nanoseconds.
		Weight::from_parts(12_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Flip Account (r:1 w:1)
	// Storage: Flip TotalIssuance (r:1 w:1)
	fn reap_one_account() -> Weight {
		// Minimum execution time: 16_000 nanoseconds.
		Weight::from_parts(17_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Flip SlashingRate (r:0 w:1)
	fn set_slashing_rate() -> Weight {
		// Minimum execution time: 11_000 nanoseconds.
		Weight::from_parts(12_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Flip Account (r:1 w:1)
	// Storage: Flip TotalIssuance (r:1 w:1)
	fn reap_one_account() -> Weight {
		// Minimum execution time: 16_000 nanoseconds.
		Weight::from_parts(17_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
}
