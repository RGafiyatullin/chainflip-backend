//! Autogenerated weights for pallet_cf_environment
//!
//! THIS FILE WAS AUTO-GENERATED USING CHAINFLIP NODE BENCHMARK CMD VERSION 4.0.0-dev
//! DATE: 2022-09-12, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// pallet_cf_environment
// --output
// state-chain/pallets/cf-environment/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_environment.
pub trait WeightInfo {
	fn set_system_state() -> Weight;
	fn set_cfe_settings() -> Weight;
	fn update_supported_eth_assets() -> Weight;
}

/// Weights for pallet_cf_environment using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Environment CurrentSystemState (r:1 w:1)
	fn set_system_state() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(26_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Environment CfeSettings (r:0 w:1)
	fn set_cfe_settings() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(23_000_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Environment SupportedEthAssets (r:0 w:1)
	fn update_supported_eth_assets() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(24_000_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Environment CurrentSystemState (r:1 w:1)
	fn set_system_state() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(26_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: Environment CfeSettings (r:0 w:1)
	fn set_cfe_settings() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(23_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: Environment SupportedEthAssets (r:0 w:1)
	fn update_supported_eth_assets() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(24_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
}
