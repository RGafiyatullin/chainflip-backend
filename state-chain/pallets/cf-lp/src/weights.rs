
//! Autogenerated weights for pallet_cf_lp
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-31, STEPS: `20`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Roys-MacBook-Pro.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/chainflip-node
// benchmark
// pallet
// --extrinsic
// *
// --pallet
// pallet_cf_lp
// --output
// state-chain/pallets/cf-lp/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=20
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_lp.
pub trait WeightInfo {
	fn request_deposit_address() -> Weight;
	fn withdraw_asset() -> Weight;
	fn register_lp_account() -> Weight;
	fn update_position() -> Weight;
}

/// Weights for pallet_cf_lp using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: EthereumIngressEgress IntentIdCounter (r:1 w:1)
	// Storage: Environment EthereumVaultAddress (r:1 w:0)
	// Storage: EthereumIngressEgress IntentActions (r:0 w:1)
	// Storage: EthereumIngressEgress IntentIngressDetails (r:0 w:1)
	fn request_deposit_address() -> Weight {
		// Minimum execution time: 36_000 nanoseconds.
		Weight::from_ref_time(41_000_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	// Storage: EthereumIngressEgress EgressIdCounter (r:1 w:1)
	// Storage: EthereumIngressEgress ScheduledEgressFetchOrTransfer (r:1 w:1)
	fn withdraw_asset() -> Weight {
		// Minimum execution time: 42_000 nanoseconds.
		Weight::from_ref_time(45_000_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: AccountRoles SwappingEnabled (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_lp_account() -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_ref_time(22_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:2 w:2)
	fn update_position() -> Weight {
		// Minimum execution time: 55_000 nanoseconds.
		Weight::from_ref_time(58_000_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: EthereumIngressEgress IntentIdCounter (r:1 w:1)
	// Storage: Environment EthereumVaultAddress (r:1 w:0)
	// Storage: EthereumIngressEgress IntentActions (r:0 w:1)
	// Storage: EthereumIngressEgress IntentIngressDetails (r:0 w:1)
	fn request_deposit_address() -> Weight {
		// Minimum execution time: 36_000 nanoseconds.
		Weight::from_ref_time(41_000_000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	// Storage: EthereumIngressEgress EgressIdCounter (r:1 w:1)
	// Storage: EthereumIngressEgress ScheduledEgressFetchOrTransfer (r:1 w:1)
	fn withdraw_asset() -> Weight {
		// Minimum execution time: 42_000 nanoseconds.
		Weight::from_ref_time(45_000_000)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: AccountRoles SwappingEnabled (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_lp_account() -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_ref_time(22_000_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:2 w:2)
	fn update_position() -> Weight {
		// Minimum execution time: 55_000 nanoseconds.
		Weight::from_ref_time(58_000_000)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
}
