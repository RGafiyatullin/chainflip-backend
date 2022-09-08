//! Autogenerated weights for pallet_cf_witnesser
//!
//! THIS FILE WAS AUTO-GENERATED USING CHAINFLIP NODE BENCHMARK CMD VERSION 4.0.0-dev
//! DATE: 2022-09-07, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("three-node-test"), DB CACHE: 1024

// Executed Command:
// ./target/release/chainflip-node
// benchmark
// pallet
// --chain
// three-node-test
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
	fn witness_at_epoch() -> Weight;
	fn remove_storage_items(n: u32, ) -> Weight;
}

/// Weights for pallet_cf_witnesser using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Validator LastExpiredEpoch (r:1 w:0)
	// Storage: Validator CurrentEpoch (r:1 w:0)
	// Storage: Validator EpochAuthorityCount (r:1 w:0)
	// Storage: Validator AuthorityIndex (r:1 w:0)
	// Storage: Witnesser Votes (r:1 w:1)
	// Storage: Witnesser CallHashExecuted (r:2 w:1)
	// Storage: Witnesser ExtraCallData (r:1 w:0)
	fn witness_at_epoch() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(38_425_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Witnesser Votes (r:0 w:1)
	fn remove_storage_items(n: u32, ) -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(46_000 as Weight)
			// Standard Error: 4_000
			.saturating_add((784_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(n as Weight)))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Validator LastExpiredEpoch (r:1 w:0)
	// Storage: Validator CurrentEpoch (r:1 w:0)
	// Storage: Validator EpochAuthorityCount (r:1 w:0)
	// Storage: Validator AuthorityIndex (r:1 w:0)
	// Storage: Witnesser Votes (r:1 w:1)
	// Storage: Witnesser CallHashExecuted (r:2 w:1)
	// Storage: Witnesser ExtraCallData (r:1 w:0)
	fn witness_at_epoch() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(38_425_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	// Storage: Witnesser Votes (r:0 w:1)
	fn remove_storage_items(n: u32, ) -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(46_000 as Weight)
			// Standard Error: 4_000
			.saturating_add((784_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(n as Weight)))
	}
}
