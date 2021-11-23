//! Autogenerated weights for pallet_cf_witnesser
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-10-26, STEPS: `[20, ]`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Interpreted, CHAIN: None, DB CACHE: 128

// Executed Command:
// ./target/release/state-chain-node
// benchmark
// --extrinsic
// *
// --pallet
// pallet_cf_witnesser
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
	fn witness() -> Weight;
}

/// Weights for pallet_cf_witnesser using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	fn witness() -> Weight {
		(2_000_000 as Weight)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn witness() -> Weight {
		(2_000_000 as Weight)
	}
}