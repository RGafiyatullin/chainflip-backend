
//! Autogenerated weights for pallet_cf_validator
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

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_validator.
pub trait WeightInfo {
	fn set_blocks_for_epoch() -> Weight;
	fn set_backup_reward_node_percentage() -> Weight;
	fn set_authority_set_min_size() -> Weight;
	fn cfe_version() -> Weight;
	fn register_peer_id() -> Weight;
	fn set_vanity_name() -> Weight;
	fn expire_epoch(a: u32, ) -> Weight;
	fn missed_authorship_slots(m: u32, ) -> Weight;
	fn rotation_phase_idle() -> Weight;
	fn start_authority_rotation(a: u32, ) -> Weight;
	fn start_authority_rotation_in_maintenance_mode() -> Weight;
	fn rotation_phase_keygen(a: u32, ) -> Weight;
	fn rotation_phase_activating_keys(a: u32, ) -> Weight;
	fn set_auction_parameters() -> Weight;
	fn register_as_validator() -> Weight;
}

/// Weights for pallet_cf_validator using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:1)
	fn set_blocks_for_epoch() -> Weight {
		// Minimum execution time: 32_573 nanoseconds.
		Weight::from_ref_time(33_487_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Validator BackupRewardNodePercentage (r:0 w:1)
	fn set_backup_reward_node_percentage() -> Weight {
		// Minimum execution time: 23_284 nanoseconds.
		Weight::from_ref_time(23_709_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuthoritySetMinSize (r:0 w:1)
	fn set_authority_set_min_size() -> Weight {
		// Minimum execution time: 30_788 nanoseconds.
		Weight::from_ref_time(31_540_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator NodeCFEVersion (r:1 w:1)
	fn cfe_version() -> Weight {
		// Minimum execution time: 40_356 nanoseconds.
		Weight::from_ref_time(40_888_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator AccountPeerMapping (r:1 w:1)
	// Storage: Validator MappedPeers (r:1 w:1)
	fn register_peer_id() -> Weight {
		// Minimum execution time: 120_304 nanoseconds.
		Weight::from_ref_time(121_964_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Validator VanityNames (r:1 w:1)
	fn set_vanity_name() -> Weight {
		// Minimum execution time: 36_854 nanoseconds.
		Weight::from_ref_time(37_831_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Validator HistoricalActiveEpochs (r:3 w:3)
	// Storage: Validator HistoricalBonds (r:1 w:0)
	// Storage: Flip Account (r:3 w:3)
	// Storage: Witnesser EpochsToCull (r:1 w:1)
	// Storage: Validator LastExpiredEpoch (r:0 w:1)
	/// The range of component `a` is `[3, 150]`.
	fn expire_epoch(a: u32, ) -> Weight {
		// Minimum execution time: 72_651 nanoseconds.
		Weight::from_ref_time(37_584_084)
			// Standard Error: 26_310
			.saturating_add(Weight::from_ref_time(13_365_588).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
	}
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:1)
	// Storage: Reputation Penalties (r:1 w:0)
	// Storage: Reputation Reputations (r:1 w:1)
	// Storage: Reputation Suspensions (r:1 w:1)
	/// The range of component `m` is `[1, 10]`.
	fn missed_authorship_slots(m: u32, ) -> Weight {
		// Minimum execution time: 52_961 nanoseconds.
		Weight::from_ref_time(46_780_997)
			// Standard Error: 552_692
			.saturating_add(Weight::from_ref_time(13_796_849).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Validator EpochExpiries (r:1 w:0)
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator CurrentEpochStartedAt (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:0)
	fn rotation_phase_idle() -> Weight {
		// Minimum execution time: 30_838 nanoseconds.
		Weight::from_ref_time(31_511_000)
			.saturating_add(T::DbWeight::get().reads(7))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuctionParameters (r:1 w:0)
	// Storage: Funding ActiveBidder (r:5 w:0)
	// Storage: Flip Account (r:4 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: Reputation LastHeartbeat (r:4 w:0)
	// Storage: Validator AccountPeerMapping (r:4 w:0)
	// Storage: Session NextKeys (r:3 w:0)
	// Storage: AccountRoles AccountRoles (r:3 w:0)
	// Storage: Validator BackupRewardNodePercentage (r:1 w:0)
	// Storage: Validator CurrentEpoch (r:1 w:0)
	// Storage: EthereumVault PendingVaultRotation (r:1 w:1)
	// Storage: EthereumVault CeremonyIdCounter (r:1 w:1)
	// Storage: PolkadotVault PendingVaultRotation (r:1 w:1)
	// Storage: PolkadotVault CeremonyIdCounter (r:1 w:1)
	// Storage: BitcoinVault PendingVaultRotation (r:1 w:1)
	// Storage: BitcoinVault CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumVault KeygenResolutionPendingSince (r:0 w:1)
	// Storage: PolkadotVault KeygenResolutionPendingSince (r:0 w:1)
	// Storage: BitcoinVault KeygenResolutionPendingSince (r:0 w:1)
	/// The range of component `a` is `[3, 400]`.
	fn start_authority_rotation(a: u32, ) -> Weight {
		// Minimum execution time: 362_287 nanoseconds.
		Weight::from_ref_time(381_931_409)
			// Standard Error: 139_604
			.saturating_add(Weight::from_ref_time(38_236_531).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(18))
			.saturating_add(T::DbWeight::get().reads((6_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	fn start_authority_rotation_in_maintenance_mode() -> Weight {
		// Minimum execution time: 13_020 nanoseconds.
		Weight::from_ref_time(13_565_000)
			.saturating_add(T::DbWeight::get().reads(1))
	}
	// Storage: Validator EpochExpiries (r:1 w:0)
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:1)
	// Storage: EthereumVault PendingVaultRotation (r:1 w:1)
	// Storage: PolkadotVault PendingVaultRotation (r:1 w:1)
	// Storage: BitcoinVault PendingVaultRotation (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: EthereumVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: PolkadotVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: BitcoinVault CurrentVaultEpochAndState (r:1 w:0)
	/// The range of component `a` is `[3, 150]`.
	fn rotation_phase_keygen(a: u32, ) -> Weight {
		// Minimum execution time: 218_503 nanoseconds.
		Weight::from_ref_time(229_840_390)
			// Standard Error: 28_466
			.saturating_add(Weight::from_ref_time(749_909).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Validator EpochExpiries (r:1 w:0)
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:1)
	// Storage: EthereumVault PendingVaultRotation (r:1 w:0)
	// Storage: PolkadotVault PendingVaultRotation (r:1 w:0)
	// Storage: BitcoinVault PendingVaultRotation (r:1 w:0)
	/// The range of component `a` is `[3, 150]`.
	fn rotation_phase_activating_keys(a: u32, ) -> Weight {
		// Minimum execution time: 153_186 nanoseconds.
		Weight::from_ref_time(169_067_581)
			// Standard Error: 30_313
			.saturating_add(Weight::from_ref_time(549_430).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuctionParameters (r:0 w:1)
	fn set_auction_parameters() -> Weight {
		// Minimum execution time: 32_395 nanoseconds.
		Weight::from_ref_time(32_933_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Flip Account (r:1 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_as_validator() -> Weight {
		// Minimum execution time: 49_759 nanoseconds.
		Weight::from_ref_time(50_278_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:1)
	fn set_blocks_for_epoch() -> Weight {
		// Minimum execution time: 32_573 nanoseconds.
		Weight::from_ref_time(33_487_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Validator BackupRewardNodePercentage (r:0 w:1)
	fn set_backup_reward_node_percentage() -> Weight {
		// Minimum execution time: 23_284 nanoseconds.
		Weight::from_ref_time(23_709_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuthoritySetMinSize (r:0 w:1)
	fn set_authority_set_min_size() -> Weight {
		// Minimum execution time: 30_788 nanoseconds.
		Weight::from_ref_time(31_540_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator NodeCFEVersion (r:1 w:1)
	fn cfe_version() -> Weight {
		// Minimum execution time: 40_356 nanoseconds.
		Weight::from_ref_time(40_888_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator AccountPeerMapping (r:1 w:1)
	// Storage: Validator MappedPeers (r:1 w:1)
	fn register_peer_id() -> Weight {
		// Minimum execution time: 120_304 nanoseconds.
		Weight::from_ref_time(121_964_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Validator VanityNames (r:1 w:1)
	fn set_vanity_name() -> Weight {
		// Minimum execution time: 36_854 nanoseconds.
		Weight::from_ref_time(37_831_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: Validator HistoricalActiveEpochs (r:3 w:3)
	// Storage: Validator HistoricalBonds (r:1 w:0)
	// Storage: Flip Account (r:3 w:3)
	// Storage: Witnesser EpochsToCull (r:1 w:1)
	// Storage: Validator LastExpiredEpoch (r:0 w:1)
	/// The range of component `a` is `[3, 150]`.
	fn expire_epoch(a: u32, ) -> Weight {
		// Minimum execution time: 72_651 nanoseconds.
		Weight::from_ref_time(37_584_084)
			// Standard Error: 26_310
			.saturating_add(Weight::from_ref_time(13_365_588).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().writes(2))
			.saturating_add(RocksDbWeight::get().writes((2_u64).saturating_mul(a.into())))
	}
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:1)
	// Storage: Reputation Penalties (r:1 w:0)
	// Storage: Reputation Reputations (r:1 w:1)
	// Storage: Reputation Suspensions (r:1 w:1)
	/// The range of component `m` is `[1, 10]`.
	fn missed_authorship_slots(m: u32, ) -> Weight {
		// Minimum execution time: 52_961 nanoseconds.
		Weight::from_ref_time(46_780_997)
			// Standard Error: 552_692
			.saturating_add(Weight::from_ref_time(13_796_849).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: Validator EpochExpiries (r:1 w:0)
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator CurrentEpochStartedAt (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:0)
	fn rotation_phase_idle() -> Weight {
		// Minimum execution time: 30_838 nanoseconds.
		Weight::from_ref_time(31_511_000)
			.saturating_add(RocksDbWeight::get().reads(7))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuctionParameters (r:1 w:0)
	// Storage: Funding ActiveBidder (r:5 w:0)
	// Storage: Flip Account (r:4 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: Reputation LastHeartbeat (r:4 w:0)
	// Storage: Validator AccountPeerMapping (r:4 w:0)
	// Storage: Session NextKeys (r:3 w:0)
	// Storage: AccountRoles AccountRoles (r:3 w:0)
	// Storage: Validator BackupRewardNodePercentage (r:1 w:0)
	// Storage: Validator CurrentEpoch (r:1 w:0)
	// Storage: EthereumVault PendingVaultRotation (r:1 w:1)
	// Storage: EthereumVault CeremonyIdCounter (r:1 w:1)
	// Storage: PolkadotVault PendingVaultRotation (r:1 w:1)
	// Storage: PolkadotVault CeremonyIdCounter (r:1 w:1)
	// Storage: BitcoinVault PendingVaultRotation (r:1 w:1)
	// Storage: BitcoinVault CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumVault KeygenResolutionPendingSince (r:0 w:1)
	// Storage: PolkadotVault KeygenResolutionPendingSince (r:0 w:1)
	// Storage: BitcoinVault KeygenResolutionPendingSince (r:0 w:1)
	/// The range of component `a` is `[3, 400]`.
	fn start_authority_rotation(a: u32, ) -> Weight {
		// Minimum execution time: 362_287 nanoseconds.
		Weight::from_ref_time(381_931_409)
			// Standard Error: 139_604
			.saturating_add(Weight::from_ref_time(38_236_531).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(18))
			.saturating_add(RocksDbWeight::get().reads((6_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().writes(10))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	fn start_authority_rotation_in_maintenance_mode() -> Weight {
		// Minimum execution time: 13_020 nanoseconds.
		Weight::from_ref_time(13_565_000)
			.saturating_add(RocksDbWeight::get().reads(1))
	}
	// Storage: Validator EpochExpiries (r:1 w:0)
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:1)
	// Storage: EthereumVault PendingVaultRotation (r:1 w:1)
	// Storage: PolkadotVault PendingVaultRotation (r:1 w:1)
	// Storage: BitcoinVault PendingVaultRotation (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: EthereumVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: PolkadotVault CurrentVaultEpochAndState (r:1 w:0)
	// Storage: BitcoinVault CurrentVaultEpochAndState (r:1 w:0)
	/// The range of component `a` is `[3, 150]`.
	fn rotation_phase_keygen(a: u32, ) -> Weight {
		// Minimum execution time: 218_503 nanoseconds.
		Weight::from_ref_time(229_840_390)
			// Standard Error: 28_466
			.saturating_add(Weight::from_ref_time(749_909).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(13))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: Validator EpochExpiries (r:1 w:0)
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:1)
	// Storage: EthereumVault PendingVaultRotation (r:1 w:0)
	// Storage: PolkadotVault PendingVaultRotation (r:1 w:0)
	// Storage: BitcoinVault PendingVaultRotation (r:1 w:0)
	/// The range of component `a` is `[3, 150]`.
	fn rotation_phase_activating_keys(a: u32, ) -> Weight {
		// Minimum execution time: 153_186 nanoseconds.
		Weight::from_ref_time(169_067_581)
			// Standard Error: 30_313
			.saturating_add(Weight::from_ref_time(549_430).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuctionParameters (r:0 w:1)
	fn set_auction_parameters() -> Weight {
		// Minimum execution time: 32_395 nanoseconds.
		Weight::from_ref_time(32_933_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Flip Account (r:1 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_as_validator() -> Weight {
		// Minimum execution time: 49_759 nanoseconds.
		Weight::from_ref_time(50_278_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
