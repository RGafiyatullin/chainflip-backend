
//! Autogenerated weights for pallet_cf_emissions
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
// pallet_cf_emissions
// --extrinsic
// *
// --output
// state-chain/pallets/cf-emissions/src/weights.rs
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

/// Weight functions needed for pallet_cf_emissions.
pub trait WeightInfo {
	fn update_backup_node_emission_inflation() -> Weight;
	fn update_current_authority_emission_inflation() -> Weight;
	fn rewards_minted() -> Weight;
	fn rewards_not_minted() -> Weight;
	fn update_supply_update_interval() -> Weight;
}

/// Weights for pallet_cf_emissions using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	/// Storage: `Emissions::BackupNodeEmissionInflation` (r:0 w:1)
	/// Proof: `Emissions::BackupNodeEmissionInflation` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn update_backup_node_emission_inflation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_717_000 picoseconds.
		Weight::from_parts(9_208_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Emissions::CurrentAuthorityEmissionInflation` (r:0 w:1)
	/// Proof: `Emissions::CurrentAuthorityEmissionInflation` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn update_current_authority_emission_inflation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_010_000 picoseconds.
		Weight::from_parts(9_436_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Emissions::CurrentAuthorityEmissionPerBlock` (r:1 w:0)
	/// Proof: `Emissions::CurrentAuthorityEmissionPerBlock` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Authorship::Author` (r:1 w:1)
	/// Proof: `Authorship::Author` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Aura::Authorities` (r:1 w:0)
	/// Proof: `Aura::Authorities` (`max_values`: Some(1), `max_size`: Some(4802), added: 5297, mode: `MaxEncodedLen`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::TotalIssuance` (r:1 w:1)
	/// Proof: `Flip::TotalIssuance` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Flip::Account` (r:1 w:1)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Emissions::SupplyUpdateInterval` (r:1 w:0)
	/// Proof: `Emissions::SupplyUpdateInterval` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Emissions::LastSupplyUpdateBlock` (r:1 w:1)
	/// Proof: `Emissions::LastSupplyUpdateBlock` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::FlipToBurn` (r:1 w:0)
	/// Proof: `LiquidityPools::FlipToBurn` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumStateChainGatewayAddress` (r:1 w:0)
	/// Proof: `Environment::EthereumStateChainGatewayAddress` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumSignatureNonce` (r:1 w:1)
	/// Proof: `Environment::EthereumSignatureNonce` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumChainId` (r:1 w:0)
	/// Proof: `Environment::EthereumChainId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumKeyManagerAddress` (r:1 w:0)
	/// Proof: `Environment::EthereumKeyManagerAddress` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumBroadcaster::BroadcastIdCounter` (r:1 w:1)
	/// Proof: `EthereumBroadcaster::BroadcastIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumChainTracking::CurrentChainState` (r:1 w:0)
	/// Proof: `EthereumChainTracking::CurrentChainState` (`max_values`: Some(1), `max_size`: Some(40), added: 535, mode: `MaxEncodedLen`)
	/// Storage: `EthereumThresholdSigner::ThresholdSignatureRequestIdCounter` (r:1 w:1)
	/// Proof: `EthereumThresholdSigner::ThresholdSignatureRequestIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CurrentKeyEpochAndState` (r:1 w:0)
	/// Proof: `EthereumVault::CurrentKeyEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::Vaults` (r:1 w:0)
	/// Proof: `EthereumVault::Vaults` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::HistoricalAuthorities` (r:1 w:0)
	/// Proof: `Validator::HistoricalAuthorities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Suspensions` (r:4 w:0)
	/// Proof: `Reputation::Suspensions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `EthereumVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::ThresholdSignatureResponseTimeout` (r:1 w:0)
	/// Proof: `EthereumThresholdSigner::ThresholdSignatureResponseTimeout` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::CeremonyRetryQueues` (r:1 w:1)
	/// Proof: `EthereumThresholdSigner::CeremonyRetryQueues` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::Signature` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::Signature` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::PendingCeremonies` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::PendingCeremonies` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::RequestCallback` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::RequestCallback` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn rewards_minted() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2187`
		//  Estimated: `13077`
		// Minimum execution time: 138_844_000 picoseconds.
		Weight::from_parts(139_946_000, 13077)
			.saturating_add(T::DbWeight::get().reads(28_u64))
			.saturating_add(T::DbWeight::get().writes(12_u64))
	}
	/// Storage: `Emissions::CurrentAuthorityEmissionPerBlock` (r:1 w:0)
	/// Proof: `Emissions::CurrentAuthorityEmissionPerBlock` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Authorship::Author` (r:1 w:1)
	/// Proof: `Authorship::Author` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Aura::Authorities` (r:1 w:0)
	/// Proof: `Aura::Authorities` (`max_values`: Some(1), `max_size`: Some(4802), added: 5297, mode: `MaxEncodedLen`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::TotalIssuance` (r:1 w:1)
	/// Proof: `Flip::TotalIssuance` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Flip::Account` (r:1 w:1)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Emissions::SupplyUpdateInterval` (r:1 w:0)
	/// Proof: `Emissions::SupplyUpdateInterval` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Emissions::LastSupplyUpdateBlock` (r:1 w:0)
	/// Proof: `Emissions::LastSupplyUpdateBlock` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn rewards_not_minted() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `979`
		//  Estimated: `6287`
		// Minimum execution time: 36_570_000 picoseconds.
		Weight::from_parts(36_825_000, 6287)
			.saturating_add(T::DbWeight::get().reads(9_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Emissions::SupplyUpdateInterval` (r:0 w:1)
	/// Proof: `Emissions::SupplyUpdateInterval` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn update_supply_update_interval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_749_000 picoseconds.
		Weight::from_parts(9_432_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Emissions::BackupNodeEmissionInflation` (r:0 w:1)
	/// Proof: `Emissions::BackupNodeEmissionInflation` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn update_backup_node_emission_inflation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_717_000 picoseconds.
		Weight::from_parts(9_208_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Emissions::CurrentAuthorityEmissionInflation` (r:0 w:1)
	/// Proof: `Emissions::CurrentAuthorityEmissionInflation` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn update_current_authority_emission_inflation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_010_000 picoseconds.
		Weight::from_parts(9_436_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Emissions::CurrentAuthorityEmissionPerBlock` (r:1 w:0)
	/// Proof: `Emissions::CurrentAuthorityEmissionPerBlock` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Authorship::Author` (r:1 w:1)
	/// Proof: `Authorship::Author` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Aura::Authorities` (r:1 w:0)
	/// Proof: `Aura::Authorities` (`max_values`: Some(1), `max_size`: Some(4802), added: 5297, mode: `MaxEncodedLen`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::TotalIssuance` (r:1 w:1)
	/// Proof: `Flip::TotalIssuance` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Flip::Account` (r:1 w:1)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Emissions::SupplyUpdateInterval` (r:1 w:0)
	/// Proof: `Emissions::SupplyUpdateInterval` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Emissions::LastSupplyUpdateBlock` (r:1 w:1)
	/// Proof: `Emissions::LastSupplyUpdateBlock` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `LiquidityPools::FlipToBurn` (r:1 w:0)
	/// Proof: `LiquidityPools::FlipToBurn` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumStateChainGatewayAddress` (r:1 w:0)
	/// Proof: `Environment::EthereumStateChainGatewayAddress` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumSignatureNonce` (r:1 w:1)
	/// Proof: `Environment::EthereumSignatureNonce` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumChainId` (r:1 w:0)
	/// Proof: `Environment::EthereumChainId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumKeyManagerAddress` (r:1 w:0)
	/// Proof: `Environment::EthereumKeyManagerAddress` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumBroadcaster::BroadcastIdCounter` (r:1 w:1)
	/// Proof: `EthereumBroadcaster::BroadcastIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumChainTracking::CurrentChainState` (r:1 w:0)
	/// Proof: `EthereumChainTracking::CurrentChainState` (`max_values`: Some(1), `max_size`: Some(40), added: 535, mode: `MaxEncodedLen`)
	/// Storage: `EthereumThresholdSigner::ThresholdSignatureRequestIdCounter` (r:1 w:1)
	/// Proof: `EthereumThresholdSigner::ThresholdSignatureRequestIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CurrentKeyEpochAndState` (r:1 w:0)
	/// Proof: `EthereumVault::CurrentKeyEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::Vaults` (r:1 w:0)
	/// Proof: `EthereumVault::Vaults` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::HistoricalAuthorities` (r:1 w:0)
	/// Proof: `Validator::HistoricalAuthorities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Suspensions` (r:4 w:0)
	/// Proof: `Reputation::Suspensions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `EthereumVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::ThresholdSignatureResponseTimeout` (r:1 w:0)
	/// Proof: `EthereumThresholdSigner::ThresholdSignatureResponseTimeout` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::CeremonyRetryQueues` (r:1 w:1)
	/// Proof: `EthereumThresholdSigner::CeremonyRetryQueues` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::Signature` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::Signature` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::PendingCeremonies` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::PendingCeremonies` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::RequestCallback` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::RequestCallback` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn rewards_minted() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2187`
		//  Estimated: `13077`
		// Minimum execution time: 138_844_000 picoseconds.
		Weight::from_parts(139_946_000, 13077)
			.saturating_add(RocksDbWeight::get().reads(28_u64))
			.saturating_add(RocksDbWeight::get().writes(12_u64))
	}
	/// Storage: `Emissions::CurrentAuthorityEmissionPerBlock` (r:1 w:0)
	/// Proof: `Emissions::CurrentAuthorityEmissionPerBlock` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Authorship::Author` (r:1 w:1)
	/// Proof: `Authorship::Author` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Aura::Authorities` (r:1 w:0)
	/// Proof: `Aura::Authorities` (`max_values`: Some(1), `max_size`: Some(4802), added: 5297, mode: `MaxEncodedLen`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::TotalIssuance` (r:1 w:1)
	/// Proof: `Flip::TotalIssuance` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Flip::Account` (r:1 w:1)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Emissions::SupplyUpdateInterval` (r:1 w:0)
	/// Proof: `Emissions::SupplyUpdateInterval` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Emissions::LastSupplyUpdateBlock` (r:1 w:0)
	/// Proof: `Emissions::LastSupplyUpdateBlock` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn rewards_not_minted() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `979`
		//  Estimated: `6287`
		// Minimum execution time: 36_570_000 picoseconds.
		Weight::from_parts(36_825_000, 6287)
			.saturating_add(RocksDbWeight::get().reads(9_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Emissions::SupplyUpdateInterval` (r:0 w:1)
	/// Proof: `Emissions::SupplyUpdateInterval` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn update_supply_update_interval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_749_000 picoseconds.
		Weight::from_parts(9_432_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
