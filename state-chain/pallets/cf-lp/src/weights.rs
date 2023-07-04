
//! Autogenerated weights for pallet_cf_lp
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
// pallet_cf_lp
// --extrinsic
// *
// --output
// state-chain/pallets/cf-lp/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
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
	// Storage: EthereumIngressEgress DepositAddressDetailsLookup (r:0 w:1)
	// Storage: EthereumIngressEgress ChannelActions (r:0 w:1)
	// Storage: EthereumIngressEgress FetchParamDetails (r:0 w:1)
	// Storage: EthereumIngressEgress AddressStatus (r:0 w:1)
	fn request_liquidity_deposit_address() -> Weight {
		// Minimum execution time: 95_916 nanoseconds.
		Weight::from_ref_time(97_069_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	// Storage: EthereumIngressEgress EgressIdCounter (r:1 w:1)
	// Storage: EthereumIngressEgress ScheduledEgressFetchOrTransfer (r:1 w:1)
	fn withdraw_asset() -> Weight {
		// Minimum execution time: 73_084 nanoseconds.
		Weight::from_ref_time(73_494_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: AccountRoles SwappingEnabled (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_lp_account() -> Weight {
		// Minimum execution time: 35_367 nanoseconds.
		Weight::from_ref_time(36_077_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: LiquidityProvider LiquidityChannelExpiries (r:1 w:1)
	// Storage: EthereumIngressEgress AddressStatus (r:1 w:0)
	// Storage: EthereumIngressEgress DepositAddressDetailsLookup (r:1 w:1)
	// Storage: EthereumIngressEgress ChannelActions (r:0 w:1)
	/// The range of component `a` is `[1, 100]`.
	fn on_initialize(a: u32, ) -> Weight {
		// Minimum execution time: 48_894 nanoseconds.
		Weight::from_ref_time(22_297_769)
			// Standard Error: 28_900
			.saturating_add(Weight::from_ref_time(22_016_491).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
	}
	// Storage: LiquidityProvider LpTTL (r:0 w:1)
	fn set_lp_ttl() -> Weight {
		// Minimum execution time: 23_501 nanoseconds.
		Weight::from_ref_time(24_221_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn register_emergency_withdrawal_address() -> Weight {
		Weight::from_ref_time(1_000_000)
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
	// Storage: EthereumIngressEgress DepositAddressDetailsLookup (r:0 w:1)
	// Storage: EthereumIngressEgress ChannelActions (r:0 w:1)
	// Storage: EthereumIngressEgress FetchParamDetails (r:0 w:1)
	// Storage: EthereumIngressEgress AddressStatus (r:0 w:1)
	fn request_liquidity_deposit_address() -> Weight {
		// Minimum execution time: 95_916 nanoseconds.
		Weight::from_ref_time(97_069_000)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	// Storage: EthereumIngressEgress EgressIdCounter (r:1 w:1)
	// Storage: EthereumIngressEgress ScheduledEgressFetchOrTransfer (r:1 w:1)
	fn withdraw_asset() -> Weight {
		// Minimum execution time: 73_084 nanoseconds.
		Weight::from_ref_time(73_494_000)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: AccountRoles SwappingEnabled (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_lp_account() -> Weight {
		// Minimum execution time: 35_367 nanoseconds.
		Weight::from_ref_time(36_077_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: LiquidityProvider LiquidityChannelExpiries (r:1 w:1)
	// Storage: EthereumIngressEgress AddressStatus (r:1 w:0)
	// Storage: EthereumIngressEgress DepositAddressDetailsLookup (r:1 w:1)
	// Storage: EthereumIngressEgress ChannelActions (r:0 w:1)
	/// The range of component `a` is `[1, 100]`.
	fn on_initialize(a: u32, ) -> Weight {
		// Minimum execution time: 48_894 nanoseconds.
		Weight::from_ref_time(22_297_769)
			// Standard Error: 28_900
			.saturating_add(Weight::from_ref_time(22_016_491).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().writes(1))
			.saturating_add(RocksDbWeight::get().writes((2_u64).saturating_mul(a.into())))
	}
	// Storage: LiquidityProvider LpTTL (r:0 w:1)
	fn set_lp_ttl() -> Weight {
		// Minimum execution time: 23_501 nanoseconds.
		Weight::from_ref_time(24_221_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn register_emergency_withdrawal_address() -> Weight {
		Weight::from_ref_time(1_000_000)
	}
}
