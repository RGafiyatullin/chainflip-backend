
//! Autogenerated weights for pallet_cf_chain_tracking
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
// pallet_cf_chain_tracking
// --extrinsic
// *
// --output
// state-chain/pallets/cf-chain-tracking/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_chain_tracking.
pub trait WeightInfo {
	fn update_chain_state() -> Weight;
}

/// Weights for pallet_cf_chain_tracking using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: EthereumChainTracking CurrentChainState (r:1 w:1)
	fn update_chain_state() -> Weight {
		// Minimum execution time: 14_000 nanoseconds.
		Weight::from_ref_time(15_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: EthereumChainTracking CurrentChainState (r:1 w:1)
	fn update_chain_state() -> Weight {
		// Minimum execution time: 14_000 nanoseconds.
		Weight::from_ref_time(15_000_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
