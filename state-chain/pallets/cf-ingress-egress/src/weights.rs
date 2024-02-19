
//! Autogenerated weights for pallet_cf_ingress_egress
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-03, STEPS: `20`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ip-172-31-9-222`, CPU: `Intel(R) Xeon(R) Platinum 8275CL CPU @ 3.00GHz`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./chainflip-node
// benchmark
// pallet
// --pallet
// pallet_cf_ingress_egress
// --extrinsic
// *
// --output
// state-chain/pallets/cf-ingress-egress/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_cf_ingress_egress.
pub trait WeightInfo {
	fn disable_asset_egress() -> Weight;
	fn process_single_deposit() -> Weight;
	fn finalise_ingress(a: u32, ) -> Weight;
	fn vault_transfer_failed() -> Weight;
	fn ccm_broadcast_failed() -> Weight;
}

/// Weights for pallet_cf_ingress_egress using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	/// Storage: `EthereumIngressEgress::DisabledEgressAssets` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::DisabledEgressAssets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn disable_asset_egress() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `74`
		//  Estimated: `3539`
		// Minimum execution time: 14_367_000 picoseconds.
		Weight::from_parts(14_878_000, 3539)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `EthereumIngressEgress::DepositChannelLookup` (r:1 w:0)
	/// Proof: `EthereumIngressEgress::DepositChannelLookup` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumIngressEgress::DepositChannelPool` (r:1 w:0)
	/// Proof: `EthereumIngressEgress::DepositChannelPool` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumIngressEgress::MinimumDeposit` (r:1 w:0)
	/// Proof: `EthereumIngressEgress::MinimumDeposit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumIngressEgress::ScheduledEgressFetchOrTransfer` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::ScheduledEgressFetchOrTransfer` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:1 w:1)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumIngressEgress::DepositBalances` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::DepositBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn process_single_deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `281`
		//  Estimated: `3746`
		// Minimum execution time: 41_785_000 picoseconds.
		Weight::from_parts(42_887_000, 3746)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `EthereumIngressEgress::DepositChannelLookup` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::DepositChannelLookup` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[1, 100]`.
	fn finalise_ingress(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `205`
		//  Estimated: `3670`
		// Minimum execution time: 2_180_000 picoseconds.
		Weight::from_parts(7_160_468, 3670)
			// Standard Error: 5_741
			.saturating_add(Weight::from_parts(1_810_382, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `EthereumIngressEgress::FailedVaultTransfers` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::FailedVaultTransfers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn vault_transfer_failed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `74`
		//  Estimated: `1559`
		// Minimum execution time: 13_462_000 picoseconds.
		Weight::from_parts(13_716_000, 1559)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn ccm_broadcast_failed() -> Weight {
		Weight::from_parts(1_000_000, 1_000)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `EthereumIngressEgress::DisabledEgressAssets` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::DisabledEgressAssets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn disable_asset_egress() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `74`
		//  Estimated: `3539`
		// Minimum execution time: 14_367_000 picoseconds.
		Weight::from_parts(14_878_000, 3539)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `EthereumIngressEgress::DepositChannelLookup` (r:1 w:0)
	/// Proof: `EthereumIngressEgress::DepositChannelLookup` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumIngressEgress::DepositChannelPool` (r:1 w:0)
	/// Proof: `EthereumIngressEgress::DepositChannelPool` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumIngressEgress::MinimumDeposit` (r:1 w:0)
	/// Proof: `EthereumIngressEgress::MinimumDeposit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumIngressEgress::ScheduledEgressFetchOrTransfer` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::ScheduledEgressFetchOrTransfer` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityProvider::FreeBalances` (r:1 w:1)
	/// Proof: `LiquidityProvider::FreeBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumIngressEgress::DepositBalances` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::DepositBalances` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn process_single_deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `281`
		//  Estimated: `3746`
		// Minimum execution time: 41_785_000 picoseconds.
		Weight::from_parts(42_887_000, 3746)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `EthereumIngressEgress::DepositChannelLookup` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::DepositChannelLookup` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[1, 100]`.
	fn finalise_ingress(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `205`
		//  Estimated: `3670`
		// Minimum execution time: 2_180_000 picoseconds.
		Weight::from_parts(7_160_468, 3670)
			// Standard Error: 5_741
			.saturating_add(Weight::from_parts(1_810_382, 0).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `EthereumIngressEgress::FailedVaultTransfers` (r:1 w:1)
	/// Proof: `EthereumIngressEgress::FailedVaultTransfers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn vault_transfer_failed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `74`
		//  Estimated: `1559`
		// Minimum execution time: 13_462_000 picoseconds.
		Weight::from_parts(13_716_000, 1559)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	fn ccm_broadcast_failed() -> Weight {
		Weight::from_parts(1_000_000, 1_000)
	}
}
