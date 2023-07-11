
//! Autogenerated weights for pallet_cf_validator
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-05, STEPS: `2`, REPEAT: 1, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `kylezs.localdomain`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/chainflip-node
// benchmark
// pallet
// --extrinsic
// *
// --pallet
// pallet_cf_validator
// --output
// state-chain/pallets/cf-validator/src/weights.rs
// --execution=wasm
// --steps=2
// --repeat=1
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
	fn start_authority_rotation_while_disabled_by_safe_mode() -> Weight;
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
		// Minimum execution time: 23_000 nanoseconds.
		Weight::from_ref_time(23_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Validator BackupRewardNodePercentage (r:0 w:1)
	fn set_backup_reward_node_percentage() -> Weight {
		// Minimum execution time: 16_000 nanoseconds.
		Weight::from_ref_time(16_000_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuthoritySetMinSize (r:0 w:1)
	fn set_authority_set_min_size() -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_ref_time(26_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator NodeCFEVersion (r:1 w:1)
	fn cfe_version() -> Weight {
		// Minimum execution time: 27_000 nanoseconds.
		Weight::from_ref_time(27_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator AccountPeerMapping (r:1 w:1)
	// Storage: Validator MappedPeers (r:1 w:1)
	fn register_peer_id() -> Weight {
		// Minimum execution time: 78_000 nanoseconds.
		Weight::from_ref_time(78_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Validator VanityNames (r:1 w:1)
	fn set_vanity_name() -> Weight {
		// Minimum execution time: 24_000 nanoseconds.
		Weight::from_ref_time(24_000_000)
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
	fn expire_epoch(_a: u32, ) -> Weight {
		// Minimum execution time: 51_000 nanoseconds.
		Weight::from_ref_time(1_430_000_000)
			.saturating_add(T::DbWeight::get().reads(303))
			.saturating_add(T::DbWeight::get().writes(302))
	}
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:1)
	// Storage: Reputation Penalties (r:1 w:0)
	// Storage: Reputation Reputations (r:1 w:1)
	// Storage: Reputation Suspensions (r:1 w:1)
	/// The range of component `m` is `[1, 10]`.
	fn missed_authorship_slots(_m: u32, ) -> Weight {
		// Minimum execution time: 47_000 nanoseconds.
		Weight::from_ref_time(142_000_000)
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
		// Minimum execution time: 23_000 nanoseconds.
		Weight::from_ref_time(23_000_000)
			.saturating_add(T::DbWeight::get().reads(7))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuctionParameters (r:1 w:0)
	// Storage: Funding ActiveBidder (r:9 w:0)
	// Storage: Flip Account (r:8 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: Reputation LastHeartbeat (r:8 w:0)
	// Storage: Validator AccountPeerMapping (r:4 w:0)
	// Storage: Session NextKeys (r:3 w:0)
	// Storage: AccountRoles AccountRoles (r:3 w:0)
	// Storage: Validator BackupRewardNodePercentage (r:1 w:0)
	// Storage: Validator CurrentEpoch (r:1 w:0)
	// Storage: EthereumVault PendingVaultRotation (r:1 w:1)
	// Storage: Validator CeremonyIdCounter (r:1 w:1)
	// Storage: PolkadotVault PendingVaultRotation (r:1 w:1)
	// Storage: BitcoinVault PendingVaultRotation (r:1 w:1)
	// Storage: EthereumVault KeygenResolutionPendingSince (r:0 w:1)
	// Storage: PolkadotVault KeygenResolutionPendingSince (r:0 w:1)
	// Storage: BitcoinVault KeygenResolutionPendingSince (r:0 w:1)
	/// The range of component `a` is `[3, 400]`.
	fn start_authority_rotation(_a: u32, ) -> Weight {
		// Minimum execution time: 333_000 nanoseconds.
		Weight::from_ref_time(11_434_000_000)
			.saturating_add(T::DbWeight::get().reads(2422))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	fn start_authority_rotation_while_disabled_by_safe_mode() -> Weight {
		// Minimum execution time: 13_000 nanoseconds.
		Weight::from_ref_time(13_000_000)
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
	fn rotation_phase_keygen(_a: u32, ) -> Weight {
		// Minimum execution time: 95_000 nanoseconds.
		Weight::from_ref_time(162_000_000)
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
	fn rotation_phase_activating_keys(_a: u32, ) -> Weight {
		// Minimum execution time: 57_000 nanoseconds.
		Weight::from_ref_time(76_000_000)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuctionParameters (r:0 w:1)
	fn set_auction_parameters() -> Weight {
		// Minimum execution time: 28_000 nanoseconds.
		Weight::from_ref_time(28_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Flip Account (r:1 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_as_validator() -> Weight {
		// Minimum execution time: 34_000 nanoseconds.
		Weight::from_ref_time(34_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Validator CurrentRotationPhase (r:1 w:0)
	// Storage: Validator BlocksPerEpoch (r:1 w:1)
	fn set_blocks_for_epoch() -> Weight {
		// Minimum execution time: 23_000 nanoseconds.
		Weight::from_ref_time(23_000_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Validator BackupRewardNodePercentage (r:0 w:1)
	fn set_backup_reward_node_percentage() -> Weight {
		// Minimum execution time: 16_000 nanoseconds.
		Weight::from_ref_time(16_000_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuthoritySetMinSize (r:0 w:1)
	fn set_authority_set_min_size() -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_ref_time(26_000_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator NodeCFEVersion (r:1 w:1)
	fn cfe_version() -> Weight {
		// Minimum execution time: 27_000 nanoseconds.
		Weight::from_ref_time(27_000_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AccountRoles AccountRoles (r:1 w:0)
	// Storage: Validator AccountPeerMapping (r:1 w:1)
	// Storage: Validator MappedPeers (r:1 w:1)
	fn register_peer_id() -> Weight {
		// Minimum execution time: 78_000 nanoseconds.
		Weight::from_ref_time(78_000_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Validator VanityNames (r:1 w:1)
	fn set_vanity_name() -> Weight {
		// Minimum execution time: 24_000 nanoseconds.
		Weight::from_ref_time(24_000_000)
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
	fn expire_epoch(_a: u32, ) -> Weight {
		// Minimum execution time: 51_000 nanoseconds.
		Weight::from_ref_time(1_430_000_000)
			.saturating_add(RocksDbWeight::get().reads(303))
			.saturating_add(RocksDbWeight::get().writes(302))
	}
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: unknown [0xac56b214382d772914db46f9c4a772eda7d533d63f25202626db56d673717380] (r:1 w:1)
	// Storage: Reputation Penalties (r:1 w:0)
	// Storage: Reputation Reputations (r:1 w:1)
	// Storage: Reputation Suspensions (r:1 w:1)
	/// The range of component `m` is `[1, 10]`.
	fn missed_authorship_slots(_m: u32, ) -> Weight {
		// Minimum execution time: 47_000 nanoseconds.
		Weight::from_ref_time(142_000_000)
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
		// Minimum execution time: 23_000 nanoseconds.
		Weight::from_ref_time(23_000_000)
			.saturating_add(RocksDbWeight::get().reads(7))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	// Storage: Validator CurrentRotationPhase (r:1 w:1)
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuctionParameters (r:1 w:0)
	// Storage: Funding ActiveBidder (r:9 w:0)
	// Storage: Flip Account (r:8 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: Reputation LastHeartbeat (r:8 w:0)
	// Storage: Validator AccountPeerMapping (r:4 w:0)
	// Storage: Session NextKeys (r:3 w:0)
	// Storage: AccountRoles AccountRoles (r:3 w:0)
	// Storage: Validator BackupRewardNodePercentage (r:1 w:0)
	// Storage: Validator CurrentEpoch (r:1 w:0)
	// Storage: EthereumVault PendingVaultRotation (r:1 w:1)
	// Storage: Validator CeremonyIdCounter (r:1 w:1)
	// Storage: PolkadotVault PendingVaultRotation (r:1 w:1)
	// Storage: BitcoinVault PendingVaultRotation (r:1 w:1)
	// Storage: EthereumVault KeygenResolutionPendingSince (r:0 w:1)
	// Storage: PolkadotVault KeygenResolutionPendingSince (r:0 w:1)
	// Storage: BitcoinVault KeygenResolutionPendingSince (r:0 w:1)
	/// The range of component `a` is `[3, 400]`.
	fn start_authority_rotation(_a: u32, ) -> Weight {
		// Minimum execution time: 333_000 nanoseconds.
		Weight::from_ref_time(11_434_000_000)
			.saturating_add(RocksDbWeight::get().reads(2422))
			.saturating_add(RocksDbWeight::get().writes(8))
	}
	// Storage: Environment CurrentSystemState (r:1 w:0)
	fn start_authority_rotation_while_disabled_by_safe_mode() -> Weight {
		// Minimum execution time: 13_000 nanoseconds.
		Weight::from_ref_time(13_000_000)
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
	fn rotation_phase_keygen(_a: u32, ) -> Weight {
		// Minimum execution time: 95_000 nanoseconds.
		Weight::from_ref_time(162_000_000)
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
	fn rotation_phase_activating_keys(_a: u32, ) -> Weight {
		// Minimum execution time: 57_000 nanoseconds.
		Weight::from_ref_time(76_000_000)
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Validator CurrentAuthorities (r:1 w:0)
	// Storage: Validator AuctionParameters (r:0 w:1)
	fn set_auction_parameters() -> Weight {
		// Minimum execution time: 28_000 nanoseconds.
		Weight::from_ref_time(28_000_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Flip Account (r:1 w:0)
	// Storage: Validator Backups (r:1 w:0)
	// Storage: AccountRoles AccountRoles (r:1 w:1)
	fn register_as_validator() -> Weight {
		// Minimum execution time: 34_000 nanoseconds.
		Weight::from_ref_time(34_000_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
