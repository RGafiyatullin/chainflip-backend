use cf_chains::{
	address::AddressDerivationApi,
	assets::sol::Asset,
	sol::{DepositChannelState, DerivedAddressBuilder},
	Solana,
};

use crate::{Environment, SolanaChainTracking};

use super::AddressDerivation;

const VAULT_PDA_ASSET_SOL: &str = "VaultPdaAssetSol";

impl AddressDerivationApi<Solana> for AddressDerivation {
	fn generate_address(
		source_asset: <Solana as cf_chains::Chain>::ChainAsset,
		channel_id: cf_primitives::ChannelId,
	) -> Result<
		<Solana as cf_chains::Chain>::ChainAccount,
		cf_chains::address::AddressDerivationError,
	> {
		let (address, _) = <Self as AddressDerivationApi<Solana>>::generate_address_and_state(
			source_asset,
			channel_id,
		)?;
		Ok(address)
	}

	fn generate_address_and_state(
		source_asset: <Solana as cf_chains::Chain>::ChainAsset,
		channel_id: cf_primitives::ChannelId,
	) -> Result<
		(
			<Solana as cf_chains::Chain>::ChainAccount,
			<Solana as cf_chains::Chain>::DepositChannelState,
		),
		cf_chains::address::AddressDerivationError,
	> {
		let vault_address = Environment::sol_vault_address();
		match source_asset {
			Asset::Sol => {
				let (pda, _bump) = DerivedAddressBuilder::from_address(vault_address)?
					.chain_seed(VAULT_PDA_ASSET_SOL)?
					.chain_seed(channel_id.to_ne_bytes())?
					.finish()?;
				log::warn!(
					"SOL DERIVED ADDR [vault: {}; CHAN: {}]: {}",
					vault_address,
					channel_id,
					pda
				);

				let last_tracked_slot =
					SolanaChainTracking::chain_state().expect("get chain_state").block_height;
				let channel_state =
					DepositChannelState { active_since_slot_number: last_tracked_slot };

				Ok((pda, channel_state))
			},
		}
	}
}
