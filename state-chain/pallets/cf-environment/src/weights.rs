
//! Autogenerated weights for pallet_cf_environment
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
// pallet_cf_environment
// --extrinsic
// *
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
	fn update_polkadot_runtime_version() -> Weight;
}

/// Weights for pallet_cf_environment using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Environment CurrentSystemState (r:1 w:1)
	fn set_system_state() -> Weight {
		// Minimum execution time: 31_372 nanoseconds.
		Weight::from_ref_time(31_969_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Environment CfeSettings (r:0 w:1)
	fn set_cfe_settings() -> Weight {
		// Minimum execution time: 24_320 nanoseconds.
		Weight::from_ref_time(24_850_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Environment EthereumSupportedAssets (r:1 w:1)
	fn update_supported_eth_assets() -> Weight {
		// Minimum execution time: 34_740 nanoseconds.
		Weight::from_ref_time(35_606_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Environment PolkadotRuntimeVersion (r:1 w:1)
	fn update_polkadot_runtime_version() -> Weight {
		// Minimum execution time: 31_403 nanoseconds.
		Weight::from_ref_time(31_827_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Environment CurrentSystemState (r:1 w:1)
	fn set_system_state() -> Weight {
		// Minimum execution time: 31_372 nanoseconds.
		Weight::from_ref_time(31_969_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Environment CfeSettings (r:0 w:1)
	fn set_cfe_settings() -> Weight {
		// Minimum execution time: 24_320 nanoseconds.
		Weight::from_ref_time(24_850_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Environment EthereumSupportedAssets (r:1 w:1)
	fn update_supported_eth_assets() -> Weight {
		// Minimum execution time: 34_740 nanoseconds.
		Weight::from_ref_time(35_606_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Environment PolkadotRuntimeVersion (r:1 w:1)
	fn update_polkadot_runtime_version() -> Weight {
		// Minimum execution time: 31_403 nanoseconds.
		Weight::from_ref_time(31_827_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
