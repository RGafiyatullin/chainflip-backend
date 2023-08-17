
//! Autogenerated weights for pallet_cf_lp
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-21, STEPS: `20`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
	fn request_liquidity_deposit_address() -> Weight;
	fn withdraw_asset() -> Weight;
	fn register_lp_account() -> Weight;
	fn on_initialize(a: u32, ) -> Weight;
	fn set_lp_ttl() -> Weight;
	fn register_emergency_withdrawal_address() -> Weight;
}

/// Weights for pallet_cf_lp using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: EthereumIngressEgress AddressPool (r:1 w:0)
	// Storage: EthereumIngressEgress ChannelIdCounter (r:1 w:1)
	// Storage: Environment EthereumVaultAddress (r:1 w:0)
	// Storage: LiquidityProvider LpTTL (r:1 w:0)
	// Storage: LiquidityProvider LiquidityChannelExpiries (r:1 w:1)
	// Storage: EthereumIngressEgress ChannelActions (r:0 w:1)
	// Storage: EthereumIngressEgress FetchParamDetails (r:0 w:1)
	// Storage: EthereumIngressEgress AddressStatus (r:0 w:1)
	// Storage: EthereumIngressEgress DepositAddressDetailsLookup (r:0 w:1)
	fn request_liquidity_deposit_address() -> Weight {
		// Minimum execution time: 55_000 nanoseconds.
		Weight::from_parts(56_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	// Storage: EthereumIngressEgress EgressIdCounter (r:1 w:1)
	// Storage: EthereumIngressEgress ScheduledEgressFetchOrTransfer (r:1 w:1)
	fn withdraw_asset() -> Weight {
		// Minimum execution time: 44_000 nanoseconds.
		Weight::from_parts(45_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: AccountRoles SwappingEnabled (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_lp_account() -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_parts(21_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: LiquidityProvider LiquidityChannelExpiries (r:1 w:1)
	// Storage: EthereumIngressEgress AddressStatus (r:1 w:0)
	// Storage: EthereumIngressEgress DepositAddressDetailsLookup (r:1 w:1)
	// Storage: EthereumIngressEgress ChannelActions (r:0 w:1)
	/// The range of component `a` is `[1, 100]`.
	fn on_initialize(a: u32, ) -> Weight {
		// Minimum execution time: 29_000 nanoseconds.
		Weight::from_parts(34_520_228, 0)
			// Standard Error: 40_282
			.saturating_add(Weight::from_parts(14_834_261, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
	}
	// Storage: LiquidityProvider LpTTL (r:0 w:1)
	fn set_lp_ttl() -> Weight {
		// Minimum execution time: 13_000 nanoseconds.
		Weight::from_parts(14_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn register_emergency_withdrawal_address() -> Weight {
		Weight::from_parts(1_000_000, 0)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: EthereumIngressEgress AddressPool (r:1 w:0)
	// Storage: EthereumIngressEgress ChannelIdCounter (r:1 w:1)
	// Storage: Environment EthereumVaultAddress (r:1 w:0)
	// Storage: LiquidityProvider LpTTL (r:1 w:0)
	// Storage: LiquidityProvider LiquidityChannelExpiries (r:1 w:1)
	// Storage: EthereumIngressEgress ChannelActions (r:0 w:1)
	// Storage: EthereumIngressEgress FetchParamDetails (r:0 w:1)
	// Storage: EthereumIngressEgress AddressStatus (r:0 w:1)
	// Storage: EthereumIngressEgress DepositAddressDetailsLookup (r:0 w:1)
	fn request_liquidity_deposit_address() -> Weight {
		// Minimum execution time: 55_000 nanoseconds.
		Weight::from_parts(56_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	// Storage: EthereumIngressEgress EgressIdCounter (r:1 w:1)
	// Storage: EthereumIngressEgress ScheduledEgressFetchOrTransfer (r:1 w:1)
	fn withdraw_asset() -> Weight {
		// Minimum execution time: 44_000 nanoseconds.
		Weight::from_parts(45_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: AccountRoles SwappingEnabled (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_lp_account() -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_parts(21_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: LiquidityProvider LiquidityChannelExpiries (r:1 w:1)
	// Storage: EthereumIngressEgress AddressStatus (r:1 w:0)
	// Storage: EthereumIngressEgress DepositAddressDetailsLookup (r:1 w:1)
	// Storage: EthereumIngressEgress ChannelActions (r:0 w:1)
	/// The range of component `a` is `[1, 100]`.
	fn on_initialize(a: u32, ) -> Weight {
		// Minimum execution time: 29_000 nanoseconds.
		Weight::from_parts(34_520_228, 0)
			// Standard Error: 40_282
			.saturating_add(Weight::from_parts(14_834_261, 0).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().writes(1))
			.saturating_add(RocksDbWeight::get().writes((2_u64).saturating_mul(a.into())))
	}
	// Storage: LiquidityProvider LpTTL (r:0 w:1)
	fn set_lp_ttl() -> Weight {
		// Minimum execution time: 13_000 nanoseconds.
		Weight::from_parts(14_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn register_emergency_withdrawal_address() -> Weight {
		Weight::from_parts(1_000_000, 0)
	}
}
