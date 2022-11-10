#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../../cf-doc-head.md")]

// This should be instatiable to the INCOMING chain.
// This way intents and intent ids align per chain, which makes sense given they act as an index to
// the respective address generation function.

use cf_primitives::{
	chains::assets::eth, Asset, AssetAmount, ForeignChain, ForeignChainAddress, IntentId,
};
use cf_traits::{liquidity::LpProvisioningApi, AddressDerivationApi, IngressApi, IngressFetchApi};

use cf_traits::SwapIntentHandler;
use frame_support::{
	pallet_prelude::*,
	sp_runtime::{app_crypto::sp_core, DispatchError},
};
use sp_std::vec;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use pallet::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use core::marker::PhantomData;

	use super::*;
	use cf_chains::Ethereum;
	use cf_traits::SwapIntentHandler;
	use frame_support::{
		pallet_prelude::{DispatchResultWithPostInfo, OptionQuery, ValueQuery},
		traits::{EnsureOrigin, IsType},
	};
	use sp_core::H256;
	use sp_std::vec::Vec;

	use frame_system::pallet_prelude::OriginFor;

	#[derive(Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, TypeInfo)]
	pub struct IngressWitness {
		pub ingress_address: ForeignChainAddress,
		pub asset: Asset,
		pub amount: u128,
		pub tx_hash: H256,
	}

	/// Details used to determine the ingress of funds.
	#[derive(Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, TypeInfo)]
	pub struct IngressDetails {
		pub intent_id: IntentId,
		pub ingress_asset: Asset,
	}

	/// Contains information relevant to the action to commence once ingress succeeds.
	#[derive(Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, TypeInfo)]
	pub enum IntentAction<AccountId> {
		Swap {
			egress_asset: Asset,
			egress_address: ForeignChainAddress,
			relayer_id: AccountId,
			relayer_commission_bps: u16,
		},
		LiquidityProvision {
			lp_account: AccountId,
		},
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	pub type IntentIngressDetails<T: Config> =
		StorageMap<_, Twox64Concat, ForeignChainAddress, IngressDetails, OptionQuery>;

	#[pallet::storage]
	pub type IntentActions<T: Config> = StorageMap<
		_,
		Twox64Concat,
		ForeignChainAddress,
		IntentAction<<T as frame_system::Config>::AccountId>,
		OptionQuery,
	>;

	/// Stores the latest intent id used to generate an address.
	#[pallet::storage]
	pub type IntentIdCounter<T: Config> = StorageValue<_, IntentId, ValueQuery>;

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: cf_traits::Chainflip {
		/// Standard Event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Generates ingress addresses.
		type AddressDerivation: AddressDerivationApi;
		/// Pallet responsible for managing Liquidity Providers.
		type LpAccountHandler: LpProvisioningApi<AccountId = Self::AccountId, Amount = AssetAmount>;
		/// For scheduling fetch requests.
		type IngressFetchApi: IngressFetchApi<Ethereum>;
		/// For scheduling swaps.
		type SwapIntentHandler: SwapIntentHandler<AccountId = Self::AccountId>;
		/// Benchmark weights
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// We only want to witness for one asset on a particular chain
		StartWitnessing {
			ingress_address: ForeignChainAddress,
			ingress_asset: Asset,
		},

		IngressCompleted {
			ingress_address: ForeignChainAddress,
			asset: Asset,
			amount: u128,
			tx_hash: H256,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidIntent,
		IngressMismatchWithIntent,
		IntentIdsExhausted,
		UnsupportedAsset,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::do_single_ingress().saturating_mul(ingress_witnesses.
		len() as u64))]
		pub fn do_ingress(
			origin: OriginFor<T>,
			ingress_witnesses: Vec<IngressWitness>,
		) -> DispatchResultWithPostInfo {
			T::EnsureWitnessed::ensure_origin(origin)?;

			for IngressWitness { ingress_address, asset, amount, tx_hash } in ingress_witnesses {
				Self::do_single_ingress(ingress_address, asset, amount, tx_hash)?;
			}
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn generate_new_address(
		ingress_asset: Asset,
	) -> Result<(IntentId, ForeignChainAddress), DispatchError> {
		let next_intent_id = IntentIdCounter::<T>::get()
			.checked_add(1)
			.ok_or(Error::<T>::IntentIdsExhausted)?;
		let ingress_address =
			T::AddressDerivation::generate_address(ingress_asset, next_intent_id)?;
		IntentIdCounter::<T>::put(next_intent_id);
		Ok((next_intent_id, ingress_address))
	}

	fn do_single_ingress(
		ingress_address: ForeignChainAddress,
		asset: Asset,
		amount: u128,
		tx_hash: sp_core::H256,
	) -> DispatchResult {
		let ingress =
			IntentIngressDetails::<T>::get(ingress_address).ok_or(Error::<T>::InvalidIntent)?;
		ensure!(ingress.ingress_asset == asset, Error::<T>::IngressMismatchWithIntent);

		// Ingress is called by witnessers, so asset/chain combination should always be valid.
		match (eth::Asset::try_from(asset), ingress_address) {
			(Ok(eth_asset), ForeignChainAddress::Eth(_)) => {
				T::IngressFetchApi::schedule_ingress_fetch(vec![(eth_asset, ingress.intent_id)]);
				Ok(())
			},
			_ => Err(Error::<T>::UnsupportedAsset),
		}?;

		// NB: Don't take here. We should continue witnessing this address
		// even after an ingress to it has occurred.
		// https://github.com/chainflip-io/chainflip-eth-contracts/pull/226
		match IntentActions::<T>::get(ingress_address).ok_or(Error::<T>::InvalidIntent)? {
			IntentAction::LiquidityProvision { lp_account, .. } => {
				match (ingress_address, ingress.ingress_asset.into()) {
					(ForeignChainAddress::Eth(_), ForeignChain::Ethereum) => {
						T::LpAccountHandler::provision_account(&lp_account, asset, amount)?;
						Ok(())
					},
					_ => Err(Error::<T>::IngressMismatchWithIntent),
				}?;
			},
			IntentAction::Swap {
				egress_address,
				egress_asset,
				relayer_id,
				relayer_commission_bps,
			} => {
				match (ingress_address, ingress.ingress_asset.into()) {
					(ForeignChainAddress::Eth(_), ForeignChain::Ethereum) => {
						T::SwapIntentHandler::schedule_swap(
							asset,
							egress_asset,
							amount,
							egress_address,
							relayer_id,
							relayer_commission_bps,
						);
						Ok(())
					},
					_ => Err(Error::<T>::IngressMismatchWithIntent),
				}?;
			},
		}
		Self::deposit_event(Event::IngressCompleted { ingress_address, asset, amount, tx_hash });
		Ok(())
	}
}

impl<T: Config> IngressApi for Pallet<T> {
	type AccountId = <T as frame_system::Config>::AccountId;

	// This should be callable by the LP pallet.
	fn register_liquidity_ingress_intent(
		lp_account: Self::AccountId,
		ingress_asset: Asset,
	) -> Result<(IntentId, ForeignChainAddress), DispatchError> {
		let (intent_id, ingress_address) = Self::generate_new_address(ingress_asset)?;

		IntentIngressDetails::<T>::insert(
			ingress_address,
			IngressDetails { intent_id, ingress_asset },
		);
		IntentActions::<T>::insert(
			ingress_address,
			IntentAction::LiquidityProvision { lp_account },
		);

		Self::deposit_event(Event::StartWitnessing { ingress_address, ingress_asset });

		Ok((intent_id, ingress_address))
	}

	// This should only be callable by the relayer.
	fn register_swap_intent(
		ingress_asset: Asset,
		egress_asset: Asset,
		egress_address: ForeignChainAddress,
		relayer_commission_bps: u16,
		relayer_id: T::AccountId,
	) -> Result<(IntentId, ForeignChainAddress), DispatchError> {
		let (intent_id, ingress_address) = Self::generate_new_address(ingress_asset)?;

		IntentIngressDetails::<T>::insert(
			ingress_address,
			IngressDetails { intent_id, ingress_asset },
		);
		IntentActions::<T>::insert(
			ingress_address,
			IntentAction::Swap { egress_address, egress_asset, relayer_commission_bps, relayer_id },
		);

		Self::deposit_event(Event::StartWitnessing { ingress_address, ingress_asset });

		Ok((intent_id, ingress_address))
	}
}
