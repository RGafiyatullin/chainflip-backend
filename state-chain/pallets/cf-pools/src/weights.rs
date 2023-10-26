
//! Autogenerated weights for pallet_cf_pools
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-10-17, STEPS: `20`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `MacBook-Pro.localdomain`, CPU: `<UNKNOWN>`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

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
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_cf_pools.
pub trait WeightInfo {
	fn update_buy_interval() -> Weight;
	fn update_pool_enabled() -> Weight;
	fn new_pool() -> Weight;
	fn update_range_order() -> Weight;
	fn set_range_order() -> Weight;
	fn update_limit_order() -> Weight;
	fn set_limit_order() -> Weight;
	fn set_pool_fees() -> Weight;
}

/// Weights for pallet_cf_pools using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	/// Storage: `LiquidityPools::FlipBuyInterval` (r:0 w:1)
	/// Proof: `LiquidityPools::FlipBuyInterval` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_buy_interval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_000_000 picoseconds.
		Weight::from_parts(8_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `LiquidityPools::Pools` (r:1 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_pool_enabled() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `418`
		//  Estimated: `3883`
		// Minimum execution time: 15_000_000 picoseconds.
		Weight::from_parts(16_000_000, 3883)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `LiquidityPools::Pools` (r:1 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn new_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `32`
		//  Estimated: `3497`
		// Minimum execution time: 17_000_000 picoseconds.
		Weight::from_parts(18_000_000, 3497)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `LiquidityProvider::LiquidityRefundAddress` (r:1 w:0)
	/// Proof: `LiquidityProvider::LiquidityRefundAddress` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::Pools` (r:2 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:2 w:2)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_range_order() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1438`
		//  Estimated: `7378`
		// Minimum execution time: 71_000_000 picoseconds.
		Weight::from_parts(72_000_000, 7378)
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `LiquidityProvider::LiquidityRefundAddress` (r:1 w:0)
	/// Proof: `LiquidityProvider::LiquidityRefundAddress` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::Pools` (r:2 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:2 w:2)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_range_order() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1438`
		//  Estimated: `7378`
		// Minimum execution time: 74_000_000 picoseconds.
		Weight::from_parts(75_000_000, 7378)
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `LiquidityProvider::LiquidityRefundAddress` (r:1 w:0)
	/// Proof: `LiquidityProvider::LiquidityRefundAddress` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::Pools` (r:2 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:1 w:1)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_limit_order() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1438`
		//  Estimated: `7378`
		// Minimum execution time: 59_000_000 picoseconds.
		Weight::from_parts(60_000_000, 7378)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `LiquidityProvider::LiquidityRefundAddress` (r:1 w:0)
	/// Proof: `LiquidityProvider::LiquidityRefundAddress` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::Pools` (r:2 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:1 w:1)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_limit_order() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1438`
		//  Estimated: `7378`
		// Minimum execution time: 57_000_000 picoseconds.
		Weight::from_parts(60_000_000, 7378)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `LiquidityPools::Pools` (r:1 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:1 w:1)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_pool_fees() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1417`
		//  Estimated: `4882`
		// Minimum execution time: 50_000_000 picoseconds.
		Weight::from_parts(51_000_000, 4882)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `LiquidityPools::FlipBuyInterval` (r:0 w:1)
	/// Proof: `LiquidityPools::FlipBuyInterval` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_buy_interval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_000_000 picoseconds.
		Weight::from_parts(8_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `LiquidityPools::Pools` (r:1 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_pool_enabled() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `418`
		//  Estimated: `3883`
		// Minimum execution time: 15_000_000 picoseconds.
		Weight::from_parts(16_000_000, 3883)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `LiquidityPools::Pools` (r:1 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn new_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `32`
		//  Estimated: `3497`
		// Minimum execution time: 17_000_000 picoseconds.
		Weight::from_parts(18_000_000, 3497)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `LiquidityProvider::LiquidityRefundAddress` (r:1 w:0)
	/// Proof: `LiquidityProvider::LiquidityRefundAddress` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::Pools` (r:2 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:2 w:2)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_range_order() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1438`
		//  Estimated: `7378`
		// Minimum execution time: 71_000_000 picoseconds.
		Weight::from_parts(72_000_000, 7378)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `LiquidityProvider::LiquidityRefundAddress` (r:1 w:0)
	/// Proof: `LiquidityProvider::LiquidityRefundAddress` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::Pools` (r:2 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:2 w:2)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_range_order() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1438`
		//  Estimated: `7378`
		// Minimum execution time: 74_000_000 picoseconds.
		Weight::from_parts(75_000_000, 7378)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `LiquidityProvider::LiquidityRefundAddress` (r:1 w:0)
	/// Proof: `LiquidityProvider::LiquidityRefundAddress` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::Pools` (r:2 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:1 w:1)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_limit_order() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1438`
		//  Estimated: `7378`
		// Minimum execution time: 59_000_000 picoseconds.
		Weight::from_parts(60_000_000, 7378)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `LiquidityProvider::LiquidityRefundAddress` (r:1 w:0)
	/// Proof: `LiquidityProvider::LiquidityRefundAddress` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::Pools` (r:2 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:1 w:1)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_limit_order() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1438`
		//  Estimated: `7378`
		// Minimum execution time: 57_000_000 picoseconds.
		Weight::from_parts(60_000_000, 7378)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `LiquidityPools::Pools` (r:1 w:1)
	/// Proof: `LiquidityPools::Pools` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:1 w:1)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_pool_fees() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1417`
		//  Estimated: `4882`
		// Minimum execution time: 50_000_000 picoseconds.
		Weight::from_parts(51_000_000, 4882)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
}
