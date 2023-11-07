pub mod v5;

use cf_runtime_upgrade_utilities::VersionedMigration;

pub type PalletMigration<T> = (VersionedMigration<crate::Pallet<T>, v5::Migration<T>, 4, 5>,);
