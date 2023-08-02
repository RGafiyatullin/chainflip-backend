
//! Autogenerated weights for pallet_cf_pools
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-04, STEPS: `20`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `MacBook-Pro.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/chainflip-node
// benchmark
// pallet
// --extrinsic
// *
// --pallet
// pallet_cf_pools
// --output
// state-chain/pallets/cf-pools/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=20
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_pools.
pub trait WeightInfo {
	fn update_buy_interval() -> Weight;
	fn update_pool_enabled() -> Weight;
	fn new_pool() -> Weight;
	fn collect_and_mint_range_order() -> Weight;
	fn collect_and_burn_range_order() -> Weight;
	fn collect_and_mint_limit_order() -> Weight;
	fn collect_and_burn_limit_order() -> Weight;
}

/// Weights for pallet_cf_pools using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: LiquidityPools FlipBuyInterval (r:0 w:1)
	fn update_buy_interval() -> Weight {
		// Minimum execution time: 25_000 nanoseconds.
		Weight::ref_time(27_000_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: LiquidityPools Pools (r:1 w:1)
	fn update_pool_enabled() -> Weight {
		// Minimum execution time: 38_000 nanoseconds.
		Weight::ref_time(40_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: LiquidityPools Pools (r:1 w:1)
	fn new_pool() -> Weight {
		// Minimum execution time: 35_000 nanoseconds.
		Weight::ref_time(36_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:2 w:2)
	fn collect_and_mint_range_order() -> Weight {
		// Minimum execution time: 83_000 nanoseconds.
		Weight::ref_time(85_000_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:2 w:2)
	fn collect_and_burn_range_order() -> Weight {
		// Minimum execution time: 81_000 nanoseconds.
		Weight::ref_time(87_000_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	fn collect_and_mint_limit_order() -> Weight {
		// Minimum execution time: 69_000 nanoseconds.
		Weight::ref_time(71_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	fn collect_and_burn_limit_order() -> Weight {
		// Minimum execution time: 72_000 nanoseconds.
		Weight::ref_time(74_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: LiquidityPools FlipBuyInterval (r:0 w:1)
	fn update_buy_interval() -> Weight {
		// Minimum execution time: 25_000 nanoseconds.
		Weight::ref_time(27_000_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: LiquidityPools Pools (r:1 w:1)
	fn update_pool_enabled() -> Weight {
		// Minimum execution time: 38_000 nanoseconds.
		Weight::ref_time(40_000_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: LiquidityPools Pools (r:1 w:1)
	fn new_pool() -> Weight {
		// Minimum execution time: 35_000 nanoseconds.
		Weight::ref_time(36_000_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:2 w:2)
	fn collect_and_mint_range_order() -> Weight {
		// Minimum execution time: 83_000 nanoseconds.
		Weight::ref_time(85_000_000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:2 w:2)
	fn collect_and_burn_range_order() -> Weight {
		// Minimum execution time: 81_000 nanoseconds.
		Weight::ref_time(87_000_000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	fn collect_and_mint_limit_order() -> Weight {
		// Minimum execution time: 69_000 nanoseconds.
		Weight::ref_time(71_000_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityPools Pools (r:1 w:1)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	fn collect_and_burn_limit_order() -> Weight {
		// Minimum execution time: 72_000 nanoseconds.
		Weight::ref_time(74_000_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
}
