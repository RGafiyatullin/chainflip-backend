
//! Autogenerated weights for pallet_cf_validator
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
// pallet_cf_validator
// --extrinsic
// *
// --output
// state-chain/pallets/cf-validator/src/weights.rs
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

/// Weight functions needed for pallet_cf_validator.
pub trait WeightInfo {
	fn update_pallet_config() -> Weight;
	fn set_authority_set_min_size() -> Weight;
	fn set_node_cfe_version() -> Weight;
	fn cfe_version() -> Weight;
	fn register_peer_id() -> Weight;
	fn set_vanity_name() -> Weight;
	fn expire_epoch(a: u32, ) -> Weight;
	fn missed_authorship_slots(m: u32, ) -> Weight;
	fn rotation_phase_idle() -> Weight;
	fn start_authority_rotation(a: u32, ) -> Weight;
	fn start_authority_rotation_while_disabled_by_safe_mode() -> Weight;
	fn rotation_phase_keygen(a: u32, ) -> Weight;
	fn rotation_phase_activating_keys(a: u32, ) -> Weight;
	fn register_as_validator() -> Weight;
}

/// Weights for pallet_cf_validator using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	/// Storage: `Validator::CurrentAuthorities` (r:1 w:0)
	/// Proof: `Validator::CurrentAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AuctionParameters` (r:0 w:1)
	/// Proof: `Validator::AuctionParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_pallet_config() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `348`
		//  Estimated: `1833`
		// Minimum execution time: 15_171_000 picoseconds.
		Weight::from_parts(15_948_000, 1833)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuthoritySetMinSize (r:0 w:1)
	fn set_authority_set_min_size() -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_parts(26_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator NodeCFEVersion (r:1 w:1)
	fn set_node_cfe_version() -> Weight {
		// Minimum execution time: 27_000 nanoseconds.
		Weight::from_parts(27_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator NodeCFEVersion (r:1 w:1)
	fn cfe_version() -> Weight {
		// Minimum execution time: 27_000 nanoseconds.
		Weight::from_parts(27_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `Validator::AccountPeerMapping` (r:1 w:1)
	/// Proof: `Validator::AccountPeerMapping` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::MappedPeers` (r:1 w:1)
	/// Proof: `Validator::MappedPeers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn register_peer_id() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `732`
		//  Estimated: `4197`
		// Minimum execution time: 91_821_000 picoseconds.
		Weight::from_parts(92_843_000, 4197)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Validator::VanityNames` (r:1 w:1)
	/// Proof: `Validator::VanityNames` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_vanity_name() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `568`
		//  Estimated: `2053`
		// Minimum execution time: 18_739_000 picoseconds.
		Weight::from_parts(19_130_000, 2053)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Validator::HistoricalAuthorities` (r:1 w:0)
	/// Proof: `Validator::HistoricalAuthorities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::HistoricalActiveEpochs` (r:150 w:150)
	/// Proof: `Validator::HistoricalActiveEpochs` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::HistoricalBonds` (r:1 w:0)
	/// Proof: `Validator::HistoricalBonds` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::Account` (r:150 w:150)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Witnesser::EpochsToCull` (r:1 w:1)
	/// Proof: `Witnesser::EpochsToCull` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::LastExpiredEpoch` (r:0 w:1)
	/// Proof: `Validator::LastExpiredEpoch` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[3, 150]`.
	fn expire_epoch(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1270 + a * (86 ±0)`
		//  Estimated: `4682 + a * (2563 ±0)`
		// Minimum execution time: 54_617_000 picoseconds.
		Weight::from_parts(22_823_689, 4682)
			// Standard Error: 17_461
			.saturating_add(Weight::from_parts(10_324_501, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(2_u64))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
			.saturating_add(Weight::from_parts(0, 2563).saturating_mul(a.into()))
	}
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:1)
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Penalties` (r:1 w:0)
	/// Proof: `Reputation::Penalties` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Reputations` (r:1 w:1)
	/// Proof: `Reputation::Reputations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Suspensions` (r:1 w:1)
	/// Proof: `Reputation::Suspensions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[1, 10]`.
	fn missed_authorship_slots(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `841`
		//  Estimated: `4306`
		// Minimum execution time: 36_793_000 picoseconds.
		Weight::from_parts(31_962_436, 4306)
			// Standard Error: 366_338
			.saturating_add(Weight::from_parts(10_770_060, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Validator::EpochExpiries` (r:1 w:0)
	/// Proof: `Validator::EpochExpiries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Storage: `Validator::CurrentRotationPhase` (r:1 w:0)
	/// Proof: `Validator::CurrentRotationPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentEpochStartedAt` (r:1 w:0)
	/// Proof: `Validator::CurrentEpochStartedAt` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::BlocksPerEpoch` (r:1 w:0)
	/// Proof: `Validator::BlocksPerEpoch` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn rotation_phase_idle() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `738`
		//  Estimated: `4203`
		// Minimum execution time: 21_210_000 picoseconds.
		Weight::from_parts(21_792_000, 4203)
			.saturating_add(T::DbWeight::get().reads(7_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentRotationPhase` (r:1 w:1)
	/// Proof: `Validator::CurrentRotationPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentAuthorities` (r:1 w:0)
	/// Proof: `Validator::CurrentAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AuctionParameters` (r:1 w:0)
	/// Proof: `Validator::AuctionParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Funding::ActiveBidder` (r:402 w:0)
	/// Proof: `Funding::ActiveBidder` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::Account` (r:401 w:0)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Reputation::LastHeartbeat` (r:401 w:0)
	/// Proof: `Reputation::LastHeartbeat` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Suspensions` (r:2 w:0)
	/// Proof: `Reputation::Suspensions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AccountPeerMapping` (r:401 w:0)
	/// Proof: `Validator::AccountPeerMapping` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::NextKeys` (r:400 w:0)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:400 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `Validator::NodeCFEVersion` (r:400 w:0)
	/// Proof: `Validator::NodeCFEVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::MinimumReportedCfeVersion` (r:1 w:0)
	/// Proof: `Validator::MinimumReportedCfeVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AuctionBidCutoffPercentage` (r:1 w:0)
	/// Proof: `Validator::AuctionBidCutoffPercentage` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentEpoch` (r:1 w:0)
	/// Proof: `Validator::CurrentEpoch` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `EthereumVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `EthereumVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `PolkadotVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `PolkadotVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `BitcoinVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `BitcoinVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::KeygenResolutionPendingSince` (r:0 w:1)
	/// Proof: `EthereumVault::KeygenResolutionPendingSince` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::KeygenResolutionPendingSince` (r:0 w:1)
	/// Proof: `PolkadotVault::KeygenResolutionPendingSince` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::KeygenResolutionPendingSince` (r:0 w:1)
	/// Proof: `BitcoinVault::KeygenResolutionPendingSince` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[3, 400]`.
	fn start_authority_rotation(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2235 + a * (427 ±0)`
		//  Estimated: `8161 + a * (2903 ±0)`
		// Minimum execution time: 233_114_000 picoseconds.
		Weight::from_parts(235_396_000, 8161)
			// Standard Error: 69_476
			.saturating_add(Weight::from_parts(31_924_260, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(20_u64))
			.saturating_add(T::DbWeight::get().reads((7_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(10_u64))
			.saturating_add(Weight::from_parts(0, 2903).saturating_mul(a.into()))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn start_authority_rotation_while_disabled_by_safe_mode() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `372`
		//  Estimated: `1857`
		// Minimum execution time: 6_639_000 picoseconds.
		Weight::from_parts(7_137_000, 1857)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `Validator::EpochExpiries` (r:1 w:0)
	/// Proof: `Validator::EpochExpiries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Storage: `Validator::CurrentRotationPhase` (r:1 w:1)
	/// Proof: `Validator::CurrentRotationPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `EthereumVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `PolkadotVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `BitcoinVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentAuthorities` (r:1 w:0)
	/// Proof: `Validator::CurrentAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CurrentVaultEpochAndState` (r:1 w:0)
	/// Proof: `EthereumVault::CurrentVaultEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::Vaults` (r:1 w:0)
	/// Proof: `EthereumVault::Vaults` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::CurrentVaultEpochAndState` (r:1 w:0)
	/// Proof: `PolkadotVault::CurrentVaultEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::CurrentVaultEpochAndState` (r:1 w:0)
	/// Proof: `BitcoinVault::CurrentVaultEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[3, 150]`.
	fn rotation_phase_keygen(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3382 + a * (32 ±0)`
		//  Estimated: `6847 + a * (32 ±0)`
		// Minimum execution time: 130_299_000 picoseconds.
		Weight::from_parts(133_343_427, 6847)
			// Standard Error: 11_061
			.saturating_add(Weight::from_parts(354_404, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(14_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(a.into()))
	}
	/// Storage: `Validator::EpochExpiries` (r:1 w:0)
	/// Proof: `Validator::EpochExpiries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Storage: `Validator::CurrentRotationPhase` (r:1 w:1)
	/// Proof: `Validator::CurrentRotationPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::PendingVaultRotation` (r:1 w:0)
	/// Proof: `EthereumVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::PendingVaultRotation` (r:1 w:0)
	/// Proof: `PolkadotVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::PendingVaultRotation` (r:1 w:0)
	/// Proof: `BitcoinVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[3, 150]`.
	fn rotation_phase_activating_keys(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2871 + a * (32 ±0)`
		//  Estimated: `6336 + a * (32 ±0)`
		// Minimum execution time: 66_706_000 picoseconds.
		Weight::from_parts(71_990_561, 6336)
			// Standard Error: 7_118
			.saturating_add(Weight::from_parts(137_978, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(a.into()))
	}
	/// Storage: `Validator::CurrentAuthorities` (r:1 w:0)
	/// Proof: `Validator::CurrentAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AuctionParameters` (r:1 w:0)
	/// Proof: `Validator::AuctionParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:1)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	fn register_as_validator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `730`
		//  Estimated: `3498`
		// Minimum execution time: 26_532_000 picoseconds.
		Weight::from_parts(27_069_000, 3498)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Validator::CurrentAuthorities` (r:1 w:0)
	/// Proof: `Validator::CurrentAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AuctionParameters` (r:0 w:1)
	/// Proof: `Validator::AuctionParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_pallet_config() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `348`
		//  Estimated: `1833`
		// Minimum execution time: 15_171_000 picoseconds.
		Weight::from_parts(15_948_000, 1833)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuthoritySetMinSize (r:0 w:1)
	fn set_authority_set_min_size() -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_parts(26_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator NodeCFEVersion (r:1 w:1)
	fn set_node_cfe_version() -> Weight {
		// Minimum execution time: 27_000 nanoseconds.
		Weight::from_parts(27_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator NodeCFEVersion (r:1 w:1)
	fn cfe_version() -> Weight {
		// Minimum execution time: 27_000 nanoseconds.
		Weight::from_parts(27_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `Validator::AccountPeerMapping` (r:1 w:1)
	/// Proof: `Validator::AccountPeerMapping` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::MappedPeers` (r:1 w:1)
	/// Proof: `Validator::MappedPeers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn register_peer_id() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `732`
		//  Estimated: `4197`
		// Minimum execution time: 91_821_000 picoseconds.
		Weight::from_parts(92_843_000, 4197)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Validator::VanityNames` (r:1 w:1)
	/// Proof: `Validator::VanityNames` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_vanity_name() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `568`
		//  Estimated: `2053`
		// Minimum execution time: 18_739_000 picoseconds.
		Weight::from_parts(19_130_000, 2053)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Validator::HistoricalAuthorities` (r:1 w:0)
	/// Proof: `Validator::HistoricalAuthorities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::HistoricalActiveEpochs` (r:150 w:150)
	/// Proof: `Validator::HistoricalActiveEpochs` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::HistoricalBonds` (r:1 w:0)
	/// Proof: `Validator::HistoricalBonds` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::Account` (r:150 w:150)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Witnesser::EpochsToCull` (r:1 w:1)
	/// Proof: `Witnesser::EpochsToCull` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::LastExpiredEpoch` (r:0 w:1)
	/// Proof: `Validator::LastExpiredEpoch` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[3, 150]`.
	fn expire_epoch(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1270 + a * (86 ±0)`
		//  Estimated: `4682 + a * (2563 ±0)`
		// Minimum execution time: 54_617_000 picoseconds.
		Weight::from_parts(22_823_689, 4682)
			// Standard Error: 17_461
			.saturating_add(Weight::from_parts(10_324_501, 0).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
			.saturating_add(RocksDbWeight::get().writes((2_u64).saturating_mul(a.into())))
			.saturating_add(Weight::from_parts(0, 2563).saturating_mul(a.into()))
	}
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:1)
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Penalties` (r:1 w:0)
	/// Proof: `Reputation::Penalties` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Reputations` (r:1 w:1)
	/// Proof: `Reputation::Reputations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Suspensions` (r:1 w:1)
	/// Proof: `Reputation::Suspensions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[1, 10]`.
	fn missed_authorship_slots(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `841`
		//  Estimated: `4306`
		// Minimum execution time: 36_793_000 picoseconds.
		Weight::from_parts(31_962_436, 4306)
			// Standard Error: 366_338
			.saturating_add(Weight::from_parts(10_770_060, 0).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Validator::EpochExpiries` (r:1 w:0)
	/// Proof: `Validator::EpochExpiries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Storage: `Validator::CurrentRotationPhase` (r:1 w:0)
	/// Proof: `Validator::CurrentRotationPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentEpochStartedAt` (r:1 w:0)
	/// Proof: `Validator::CurrentEpochStartedAt` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::BlocksPerEpoch` (r:1 w:0)
	/// Proof: `Validator::BlocksPerEpoch` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn rotation_phase_idle() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `738`
		//  Estimated: `4203`
		// Minimum execution time: 21_210_000 picoseconds.
		Weight::from_parts(21_792_000, 4203)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentRotationPhase` (r:1 w:1)
	/// Proof: `Validator::CurrentRotationPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentAuthorities` (r:1 w:0)
	/// Proof: `Validator::CurrentAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AuctionParameters` (r:1 w:0)
	/// Proof: `Validator::AuctionParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Funding::ActiveBidder` (r:402 w:0)
	/// Proof: `Funding::ActiveBidder` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::Account` (r:401 w:0)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Reputation::LastHeartbeat` (r:401 w:0)
	/// Proof: `Reputation::LastHeartbeat` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Suspensions` (r:2 w:0)
	/// Proof: `Reputation::Suspensions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AccountPeerMapping` (r:401 w:0)
	/// Proof: `Validator::AccountPeerMapping` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::NextKeys` (r:400 w:0)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:400 w:0)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `Validator::NodeCFEVersion` (r:400 w:0)
	/// Proof: `Validator::NodeCFEVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::MinimumReportedCfeVersion` (r:1 w:0)
	/// Proof: `Validator::MinimumReportedCfeVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AuctionBidCutoffPercentage` (r:1 w:0)
	/// Proof: `Validator::AuctionBidCutoffPercentage` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentEpoch` (r:1 w:0)
	/// Proof: `Validator::CurrentEpoch` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `EthereumVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `EthereumVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `PolkadotVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `PolkadotVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `BitcoinVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `BitcoinVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::KeygenResolutionPendingSince` (r:0 w:1)
	/// Proof: `EthereumVault::KeygenResolutionPendingSince` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::KeygenResolutionPendingSince` (r:0 w:1)
	/// Proof: `PolkadotVault::KeygenResolutionPendingSince` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::KeygenResolutionPendingSince` (r:0 w:1)
	/// Proof: `BitcoinVault::KeygenResolutionPendingSince` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[3, 400]`.
	fn start_authority_rotation(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2235 + a * (427 ±0)`
		//  Estimated: `8161 + a * (2903 ±0)`
		// Minimum execution time: 233_114_000 picoseconds.
		Weight::from_parts(235_396_000, 8161)
			// Standard Error: 69_476
			.saturating_add(Weight::from_parts(31_924_260, 0).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(20_u64))
			.saturating_add(RocksDbWeight::get().reads((7_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().writes(10_u64))
			.saturating_add(Weight::from_parts(0, 2903).saturating_mul(a.into()))
	}
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn start_authority_rotation_while_disabled_by_safe_mode() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `372`
		//  Estimated: `1857`
		// Minimum execution time: 6_639_000 picoseconds.
		Weight::from_parts(7_137_000, 1857)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	/// Storage: `Validator::EpochExpiries` (r:1 w:0)
	/// Proof: `Validator::EpochExpiries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Storage: `Validator::CurrentRotationPhase` (r:1 w:1)
	/// Proof: `Validator::CurrentRotationPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `EthereumVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `PolkadotVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::PendingVaultRotation` (r:1 w:1)
	/// Proof: `BitcoinVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::RuntimeSafeMode` (r:1 w:0)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::CurrentAuthorities` (r:1 w:0)
	/// Proof: `Validator::CurrentAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CurrentVaultEpochAndState` (r:1 w:0)
	/// Proof: `EthereumVault::CurrentVaultEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::Vaults` (r:1 w:0)
	/// Proof: `EthereumVault::Vaults` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::CurrentVaultEpochAndState` (r:1 w:0)
	/// Proof: `PolkadotVault::CurrentVaultEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::CurrentVaultEpochAndState` (r:1 w:0)
	/// Proof: `BitcoinVault::CurrentVaultEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[3, 150]`.
	fn rotation_phase_keygen(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3382 + a * (32 ±0)`
		//  Estimated: `6847 + a * (32 ±0)`
		// Minimum execution time: 130_299_000 picoseconds.
		Weight::from_parts(133_343_427, 6847)
			// Standard Error: 11_061
			.saturating_add(Weight::from_parts(354_404, 0).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(14_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(a.into()))
	}
	/// Storage: `Validator::EpochExpiries` (r:1 w:0)
	/// Proof: `Validator::EpochExpiries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Digest` (r:1 w:0)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380` (r:1 w:0)
	/// Storage: `Validator::CurrentRotationPhase` (r:1 w:1)
	/// Proof: `Validator::CurrentRotationPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::PendingVaultRotation` (r:1 w:0)
	/// Proof: `EthereumVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotVault::PendingVaultRotation` (r:1 w:0)
	/// Proof: `PolkadotVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `BitcoinVault::PendingVaultRotation` (r:1 w:0)
	/// Proof: `BitcoinVault::PendingVaultRotation` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[3, 150]`.
	fn rotation_phase_activating_keys(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2871 + a * (32 ±0)`
		//  Estimated: `6336 + a * (32 ±0)`
		// Minimum execution time: 66_706_000 picoseconds.
		Weight::from_parts(71_990_561, 6336)
			// Standard Error: 7_118
			.saturating_add(Weight::from_parts(137_978, 0).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(8_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(a.into()))
	}
	/// Storage: `Validator::CurrentAuthorities` (r:1 w:0)
	/// Proof: `Validator::CurrentAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::AuctionParameters` (r:1 w:0)
	/// Proof: `Validator::AuctionParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AccountRoles::AccountRoles` (r:1 w:1)
	/// Proof: `AccountRoles::AccountRoles` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	fn register_as_validator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `730`
		//  Estimated: `3498`
		// Minimum execution time: 26_532_000 picoseconds.
		Weight::from_parts(27_069_000, 3498)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
