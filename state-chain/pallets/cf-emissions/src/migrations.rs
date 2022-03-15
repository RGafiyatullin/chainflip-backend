use super::*;
#[cfg(feature = "try-runtime")]
use frame_support::pallet_prelude::GetStorageVersion;
use frame_support::{
	traits::{OnRuntimeUpgrade, PalletInfoAccess, StorageVersion},
	weights::RuntimeDbWeight,
};
use sp_std::marker::PhantomData;

pub type StorageMigration<P, T> = VersionedMigration<P, add_mint_interval::Migration<T>, 1, 2>;

pub struct VersionedMigration<P, U, const FROM: u16, const TO: u16>(PhantomData<(P, U)>);

#[cfg(feature = "try-runtime")]
mod try_runtime_helpers {
	use frame_support::{traits::PalletInfoAccess, Twox64Concat};

	frame_support::generate_storage_alias!(
		RuntimeUpgradeUtils, ExpectMigration => Map<(Vec<u8>, Twox64Concat), bool>
	);

	pub fn expect_migration<T: PalletInfoAccess>() {
		ExpectMigration::insert(T::name().as_bytes(), true);
	}

	pub fn migration_expected<T: PalletInfoAccess>() -> bool {
		ExpectMigration::get(T::name().as_bytes()).unwrap_or_default()
	}
}

impl<P, U, const FROM: u16, const TO: u16> OnRuntimeUpgrade for VersionedMigration<P, U, FROM, TO>
where
	P: PalletInfoAccess + GetStorageVersion,
	U: OnRuntimeUpgrade,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		if <P as GetStorageVersion>::on_chain_storage_version() == FROM {
			let w = U::on_runtime_upgrade();
			StorageVersion::new(TO).put::<P>();
			w + RuntimeDbWeight::default().reads_writes(1, 1)
		} else {
			log::info!(
				"Skipping storage migration from version {:?} to {:?} - consider removing this from the runtime.",
				FROM, TO
			);
			RuntimeDbWeight::default().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		if <P as GetStorageVersion>::on_chain_storage_version() == FROM {
			try_runtime_helpers::expect_migration::<P>();
			U::pre_upgrade()
		} else {
			Ok(())
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		if !try_runtime_helpers::migration_expected::<P>() {
			return Ok(())
		}

		if <P as GetStorageVersion>::on_chain_storage_version() == TO {
			U::post_upgrade()
		} else {
			log::error!("Expected post-upgrade storage version {:?}, found {:?}.", FROM, TO);
			Err("Pallet storage migration version mismatch.")
		}
	}
}

pub(crate) mod add_mint_interval {
	use super::*;

	// The value for the MintInterval
	// runtime constant in pallet version V0
	const MINT_INTERVAL_V0: u32 = 100;

	pub struct Migration<T>(PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for Migration<T> {
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			MintInterval::<T>::put(T::BlockNumber::from(MINT_INTERVAL_V0));
			RuntimeDbWeight::default().reads_writes(0, 1)
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			Ok(())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			assert_eq!(T::BlockNumber::from(100 as u32), MintInterval::<T>::get());
			log::info!(
				target: "runtime::cf_emissions",
				"migration: Emissions storage version v1 POST migration checks successful!"
			);
			Ok(())
		}
	}
}
