#![cfg_attr(not(feature = "std"), no_std)]
#![feature(extract_if)]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../../cf-doc-head.md")]

mod benchmarking;

pub mod migrations;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;
use cf_runtime_utilities::log_or_panic;
use frame_support::{sp_runtime::SaturatedConversion, traits::OnRuntimeUpgrade};
pub use weights::WeightInfo;

use cf_chains::{
	address::{AddressConverter, AddressDerivationApi},
	AllBatch, AllBatchError, CcmCfParameters, CcmChannelMetadata, CcmDepositMetadata, CcmMessage,
	Chain, DepositChannel, DepositTracker, ExecutexSwapAndCall, ForeignChainAddress, SwapOrigin,
	TransferAssetParams,
};
use cf_primitives::{
	Asset, AssetAmount, BasisPoints, ChannelId, EgressCounter, EgressId, ForeignChain,
};
use cf_traits::{
	liquidity::LpBalanceApi, Broadcaster, CcmHandler, Chainflip, DepositApi, EgressApi,
	GetBlockHeight, SwapDepositHandler,
};
use frame_support::{
	pallet_prelude::*,
	sp_runtime::{traits::Zero, DispatchError, Saturating, TransactionOutcome},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_std::{vec, vec::Vec};

/// Enum wrapper for fetch and egress requests.
#[derive(RuntimeDebug, Eq, PartialEq, Clone, Encode, Decode, TypeInfo)]
pub struct ScheduledTransfer<C: Chain> {
	pub egress_id: EgressId,
	pub params: TransferAssetParams<C>,
}

/// Cross-chain messaging requests.
#[derive(RuntimeDebug, Eq, PartialEq, Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub(crate) struct CrossChainMessage<C: Chain> {
	pub egress_id: EgressId,
	pub asset: C::ChainAsset,
	pub amount: C::ChainAmount,
	pub destination_address: C::ChainAccount,
	pub message: CcmMessage,
	// The sender of the deposit transaction.
	pub source_chain: ForeignChain,
	pub source_address: Option<ForeignChainAddress>,
	// Where funds might be returned to if the message fails.
	pub cf_parameters: CcmCfParameters,
	pub gas_budget: C::ChainAmount,
}

impl<C: Chain> CrossChainMessage<C> {
	fn asset(&self) -> C::ChainAsset {
		self.asset
	}
}

#[derive(RuntimeDebug, Eq, PartialEq, Clone, Encode, Decode, TypeInfo)]
pub struct VaultTransfer<C: Chain> {
	asset: C::ChainAsset,
	amount: C::ChainAmount,
	destination_address: C::ChainAccount,
}

pub const PALLET_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use cf_chains::ExecutexSwapAndCall;
	use cf_primitives::BroadcastId;
	use core::marker::PhantomData;
	use frame_support::{
		storage::with_transaction,
		traits::{EnsureOrigin, IsType},
	};
	use sp_std::vec::Vec;

	pub(crate) type ChannelRecycleQueue<T, I> =
		Vec<(TargetChainBlockNumber<T, I>, TargetChainAccount<T, I>)>;

	pub(crate) type TargetChainAsset<T, I> = <<T as Config<I>>::TargetChain as Chain>::ChainAsset;
	pub(crate) type TargetChainAccount<T, I> =
		<<T as Config<I>>::TargetChain as Chain>::ChainAccount;
	pub(crate) type TargetChainAmount<T, I> = <<T as Config<I>>::TargetChain as Chain>::ChainAmount;
	pub(crate) type TargetChainBlockNumber<T, I> =
		<<T as Config<I>>::TargetChain as Chain>::ChainBlockNumber;
	pub(crate) type TargetChainDepositTracker<T, I> =
		<<T as Config<I>>::TargetChain as Chain>::DepositTracker;

	#[derive(Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub struct DepositWitness<C: Chain> {
		pub deposit_address: C::ChainAccount,
		pub asset: C::ChainAsset,
		pub amount: C::ChainAmount,
		pub deposit_details: C::DepositDetails,
	}

	#[derive(
		CloneNoBound, RuntimeDebug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen,
	)]
	#[scale_info(skip_type_params(T, I))]
	pub struct DepositChannelDetails<T: Config<I>, I: 'static> {
		pub deposit_channel: <T::TargetChain as Chain>::DepositChannel,
		/// The block number at which the deposit channel was opened, expressed as a block number
		/// on the external Chain.
		pub opened_at: TargetChainBlockNumber<T, I>,
		/// The last block on the target chain that the witnessing will witness it in. If funds are
		/// sent after this block, they will not be witnessed.
		pub expires_at: TargetChainBlockNumber<T, I>,

		/// The action to be taken when the DepositChannel is deposited to.
		pub action: ChannelAction<T::AccountId>,
	}

	/// Determines the action to take when a deposit is made to a channel.
	#[derive(Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, TypeInfo)]
	pub enum ChannelAction<AccountId> {
		Swap {
			destination_asset: Asset,
			destination_address: ForeignChainAddress,
			broker_id: AccountId,
			broker_commission_bps: BasisPoints,
		},
		LiquidityProvision {
			lp_account: AccountId,
		},
		CcmTransfer {
			destination_asset: Asset,
			destination_address: ForeignChainAddress,
			channel_metadata: CcmChannelMetadata,
		},
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		pub deposit_channel_lifetime: TargetChainBlockNumber<T, I>,
	}

	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self { deposit_channel_lifetime: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
		fn build(&self) {
			DepositChannelLifetime::<T, I>::put(self.deposit_channel_lifetime);
		}
	}

	#[pallet::pallet]
	#[pallet::storage_version(PALLET_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config<I: 'static = ()>: Chainflip {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The pallet dispatches calls, so it depends on the runtime's aggregated Call type.
		type RuntimeCall: From<Call<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeCall>;

		/// Marks which chain this pallet is interacting with.
		type TargetChain: Chain + Get<ForeignChain>;

		/// Generates deposit addresses.
		type AddressDerivation: AddressDerivationApi<Self::TargetChain>;

		/// A converter to convert address to and from human readable to internal address
		/// representation.
		type AddressConverter: AddressConverter;

		/// Pallet responsible for managing Liquidity Providers.
		type LpBalance: LpBalanceApi<AccountId = Self::AccountId>;

		/// For scheduling swaps.
		type SwapDepositHandler: SwapDepositHandler<AccountId = Self::AccountId>;

		/// Handler for Cross Chain Messages.
		type CcmHandler: CcmHandler;

		/// The type of the chain-native transaction.
		type ChainApiCall: AllBatch<Self::TargetChain> + ExecutexSwapAndCall<Self::TargetChain>;

		/// Get the latest block height of the target chain via Chain Tracking.
		type ChainTracking: GetBlockHeight<Self::TargetChain>;

		/// A broadcaster instance.
		type Broadcaster: Broadcaster<
			Self::TargetChain,
			ApiCall = Self::ChainApiCall,
			Callback = <Self as Config<I>>::RuntimeCall,
		>;

		type DepositTracker: DepositTracker<Self::TargetChain>;

		/// Benchmark weights
		type WeightInfo: WeightInfo;
	}

	/// Lookup table for addresses to correpsponding deposit channels.
	#[pallet::storage]
	pub type DepositChannelLookup<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		TargetChainAccount<T, I>,
		DepositChannelDetails<T, I>,
		OptionQuery,
	>;

	/// Stores the latest channel id used to generate an address.
	#[pallet::storage]
	pub type ChannelIdCounter<T: Config<I>, I: 'static = ()> =
		StorageValue<_, ChannelId, ValueQuery>;

	/// Stores the latest egress id used to generate an address.
	#[pallet::storage]
	pub type EgressIdCounter<T: Config<I>, I: 'static = ()> =
		StorageValue<_, EgressCounter, ValueQuery>;

	/// Scheduled transfers for the target chain.
	#[pallet::storage]
	pub(crate) type ScheduledTransfers<T: Config<I>, I: 'static = ()> =
		StorageValue<_, Vec<ScheduledTransfer<T::TargetChain>>, ValueQuery>;

	/// Scheduled cross chain messages for the Ethereum chain.
	#[pallet::storage]
	pub(crate) type ScheduledEgressCcm<T: Config<I>, I: 'static = ()> =
		StorageValue<_, Vec<CrossChainMessage<T::TargetChain>>, ValueQuery>;

	/// Stores the list of assets that are not allowed to be egressed.
	#[pallet::storage]
	pub type DisabledEgressAssets<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, TargetChainAsset<T, I>, ()>;

	/// Stores recycled addresses that can be re-used.
	#[pallet::storage]
	pub(crate) type DepositChannelPool<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Twox64Concat,
		Option<Asset>,
		Twox64Concat,
		ChannelId,
		<T::TargetChain as Chain>::DepositChannel,
	>;

	/// Defines the minimum amount of Deposit allowed for each asset.
	#[pallet::storage]
	pub type MinimumDeposit<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, TargetChainAsset<T, I>, TargetChainAmount<T, I>, ValueQuery>;

	#[pallet::storage]
	pub type DepositChannelLifetime<T: Config<I>, I: 'static = ()> =
		StorageValue<_, TargetChainBlockNumber<T, I>, ValueQuery>;

	/// Stores any failed transfers by the Vault contract.
	/// Without dealing with the underlying reason for the failure, retrying is unlike to succeed.
	/// Therefore these calls are stored here, until we can react to the reason for failure and
	/// respond appropriately.
	#[pallet::storage]
	pub type FailedVaultTransfers<T: Config<I>, I: 'static = ()> =
		StorageValue<_, Vec<VaultTransfer<T::TargetChain>>, ValueQuery>;

	#[pallet::storage]
	// TODO: rename to `Deposits`
	pub type DepositBalances<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		TargetChainAsset<T, I>,
		TargetChainDepositTracker<T, I>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type DepositChannelRecycleBlocks<T: Config<I>, I: 'static = ()> =
		StorageValue<_, ChannelRecycleQueue<T, I>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		DepositReceived {
			deposit_address: TargetChainAccount<T, I>,
			asset: TargetChainAsset<T, I>,
			amount: TargetChainAmount<T, I>,
			deposit_details: <T::TargetChain as Chain>::DepositDetails,
		},
		AssetEgressStatusChanged {
			asset: TargetChainAsset<T, I>,
			disabled: bool,
		},
		EgressScheduled {
			id: EgressId,
			asset: TargetChainAsset<T, I>,
			amount: AssetAmount,
			destination_address: TargetChainAccount<T, I>,
		},
		CcmBroadcastRequested {
			broadcast_id: BroadcastId,
			egress_id: EgressId,
		},
		CcmEgressInvalid {
			egress_id: EgressId,
			error: DispatchError,
		},
		DepositFetchesScheduled {
			channel_id: ChannelId,
			asset: TargetChainAsset<T, I>,
		},
		BatchBroadcastRequested {
			broadcast_id: BroadcastId,
			egress_ids: Vec<EgressId>,
		},
		MinimumDepositSet {
			asset: TargetChainAsset<T, I>,
			minimum_deposit: TargetChainAmount<T, I>,
		},
		///The deposits is rejected because the amount is below the minimum allowed.
		DepositIgnored {
			deposit_address: TargetChainAccount<T, I>,
			asset: TargetChainAsset<T, I>,
			amount: TargetChainAmount<T, I>,
			deposit_details: <T::TargetChain as Chain>::DepositDetails,
		},
		VaultTransferFailed {
			asset: TargetChainAsset<T, I>,
			amount: TargetChainAmount<T, I>,
			destination_address: TargetChainAccount<T, I>,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// The deposit address is not valid. It may have expired or may never have been issued.
		InvalidDepositAddress,
		/// A deposit was made using the wrong asset.
		AssetMismatch,
		/// Channel ID has reached maximum
		ChannelIdsExhausted,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		/// Recycle addresses if we can
		fn on_idle(_n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
			let read_write_weight =
				frame_support::weights::constants::RocksDbWeight::get().reads_writes(1, 1);

			let maximum_recycle_number = remaining_weight
				.ref_time()
				.checked_div(read_write_weight.ref_time())
				.unwrap_or_default()
				.saturated_into::<usize>();

			let can_recycle = DepositChannelRecycleBlocks::<T, I>::mutate(|recycle_queue| {
				Self::can_and_cannot_recycle(
					recycle_queue,
					maximum_recycle_number,
					T::ChainTracking::get_block_height(),
				)
			});

			for address in can_recycle.iter() {
				if let Some(details) = DepositChannelLookup::<T, I>::take(address) {
					if let Some(recycled_channel) = DepositBalances::<T, I>::mutate(
						details.deposit_channel.asset(),
						|deposit_tracker| {
							deposit_tracker.maybe_recycle_channel(details.deposit_channel)
						},
					) {
						DepositChannelPool::<T, I>::insert(
							recycled_channel.asset(),
							recycled_channel.channel_id(),
							channel,
						);
					}
				}
			}

			read_write_weight.saturating_mul(can_recycle.len() as u64)
		}

		/// Take all scheduled Egress and send them out
		fn on_finalize(_n: BlockNumberFor<T>) {
			// Send all fetch/transfer requests as a batch. Revert storage if failed.
			if let Err(e) = with_transaction(|| Self::do_egress_scheduled_fetch_transfer()) {
				log::error!("Ingress-Egress failed to send BatchAll. Error: {e:?}");
			}

			// Egress all scheduled Cross chain messages
			Self::do_egress_scheduled_ccm();
		}

		fn on_runtime_upgrade() -> Weight {
			migrations::PalletMigration::<T, I>::on_runtime_upgrade()
		}
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, DispatchError> {
			migrations::PalletMigration::<T, I>::pre_upgrade()
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), DispatchError> {
			migrations::PalletMigration::<T, I>::post_upgrade(state)
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Callback for when a signature is accepted by the chain.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::finalise_ingress(addresses.len() as u32))]
		pub fn finalise_ingress(
			origin: OriginFor<T>,
			addresses: Vec<TargetChainAccount<T, I>>,
		) -> DispatchResult {
			T::EnsureWitnessedAtCurrentEpoch::ensure_origin(origin)?;

			for deposit_address in addresses {
				if let Some(mut deposit_details) =
					DepositChannelLookup::<T, I>::get(&deposit_address)
				{
					if deposit_details
						.deposit_channel
						.on_fetch_completed(&deposit_details.deposit_channel)
					{
						DepositChannelLookup::<T, I>::insert(&deposit_address, deposit_details);
					}
				} else {
					log::error!(
						"Deposit address {:?} not found in DepositChannelLookup",
						deposit_address
					);
				}
			}
			Ok(())
		}

		/// Sets if an asset is not allowed to be sent out of the chain via Egress.
		/// Requires Governance
		///
		/// ## Events
		///
		/// - [On update](Event::AssetEgressStatusChanged)
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::disable_asset_egress())]
		pub fn enable_or_disable_egress(
			origin: OriginFor<T>,
			asset: TargetChainAsset<T, I>,
			set_disabled: bool,
		) -> DispatchResult {
			T::EnsureGovernance::ensure_origin(origin)?;

			let is_currently_disabled = DisabledEgressAssets::<T, I>::contains_key(asset);

			let do_disable = !is_currently_disabled && set_disabled;
			let do_enable = is_currently_disabled && !set_disabled;

			if do_disable {
				DisabledEgressAssets::<T, I>::insert(asset, ());
			} else if do_enable {
				DisabledEgressAssets::<T, I>::remove(asset);
			}

			if do_disable || do_enable {
				Self::deposit_event(Event::<T, I>::AssetEgressStatusChanged {
					asset,
					disabled: set_disabled,
				});
			}

			Ok(())
		}

		/// Called when funds have been deposited into the given address.
		///
		/// Requires `EnsureWitnessed` origin.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::process_single_deposit().saturating_mul(deposit_witnesses.len() as u64))]
		pub fn process_deposits(
			origin: OriginFor<T>,
			deposit_witnesses: Vec<DepositWitness<T::TargetChain>>,
			block_height: TargetChainBlockNumber<T, I>,
		) -> DispatchResult {
			T::EnsureWitnessed::ensure_origin(origin)?;

			for DepositWitness { deposit_address, asset, amount, deposit_details } in
				deposit_witnesses
			{
				Self::process_single_deposit(
					deposit_address,
					asset,
					amount,
					deposit_details,
					block_height,
				)?;
			}
			Ok(())
		}

		/// Sets the minimum deposit amount allowed for an asset.
		/// Requires governance
		///
		/// ## Events
		///
		/// - [on_success](Event::MinimumDepositSet)
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::set_minimum_deposit())]
		pub fn set_minimum_deposit(
			origin: OriginFor<T>,
			asset: TargetChainAsset<T, I>,
			minimum_deposit: TargetChainAmount<T, I>,
		) -> DispatchResult {
			T::EnsureGovernance::ensure_origin(origin)?;

			MinimumDeposit::<T, I>::insert(asset, minimum_deposit);

			Self::deposit_event(Event::<T, I>::MinimumDepositSet { asset, minimum_deposit });
			Ok(())
		}

		/// Stores information on failed Vault transfer.
		/// Requires Witness origin.
		///
		/// ## Events
		///
		/// - [on_success](Event::VaultTransferFailed)
		#[pallet::weight(T::WeightInfo::vault_transfer_failed())]
		#[pallet::call_index(4)]
		pub fn vault_transfer_failed(
			origin: OriginFor<T>,
			asset: TargetChainAsset<T, I>,
			amount: TargetChainAmount<T, I>,
			destination_address: TargetChainAccount<T, I>,
		) -> DispatchResult {
			T::EnsureWitnessed::ensure_origin(origin)?;

			FailedVaultTransfers::<T, I>::append(VaultTransfer {
				asset,
				amount,
				destination_address: destination_address.clone(),
			});

			Self::deposit_event(Event::<T, I>::VaultTransferFailed {
				asset,
				amount,
				destination_address,
			});
			Ok(())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	fn can_and_cannot_recycle(
		channel_recycle_blocks: &mut ChannelRecycleQueue<T, I>,
		maximum_recyclable_number: usize,
		current_block_height: TargetChainBlockNumber<T, I>,
	) -> Vec<TargetChainAccount<T, I>> {
		let partition_point = sp_std::cmp::min(
			channel_recycle_blocks.partition_point(|(block, _)| *block <= current_block_height),
			maximum_recyclable_number,
		);
		channel_recycle_blocks
			.drain(..partition_point)
			.map(|(_, address)| address)
			.collect()
	}

	/// Take all scheduled egress requests and send them out in an `AllBatch` call.
	///
	/// Note: Egress transactions with Blacklisted assets are not sent, and kept in storage.
	fn do_egress_scheduled_fetch_transfer() -> TransactionOutcome<DispatchResult> {
		let batch_to_send: Vec<_> = ScheduledTransfers::<T, I>::mutate(|transfers: &mut Vec<_>| {
			// Filter out disabled assets.
			transfers
				.extract_if(|ScheduledTransfer { params, .. }| {
					!DisabledEgressAssets::<T, I>::contains_key(params.asset)
				})
				.collect()
		});

		if batch_to_send.is_empty() {
			return TransactionOutcome::Commit(Ok(()))
		}

		let total_transfer_amounts = batch_to_send.iter().fold(
			enum_map::enum_map! { _ => Zero::zero() },
			|mut totals, transfer| {
				totals[transfer.params.asset].saturating_accrue(transfer.params.amount);
				totals
			},
		);

		let fetch_params = total_transfer_amounts
			.iter()
			.map(|(asset, transfer_amount)| {
				// TODO: use fetched amount to add change amount.
				DepositBalances::<T, I>::mutate(asset, |tracker| {
					tracker.withdraw_at_least(
						*transfer_amount,
						todo!("fees"),
						DepositChannelLookup::<T, I>::mutate,
					)
				})
				.map(|(fetches, _)| fetches)
			})
			.flatten()
			.collect::<Vec<_>>();

		let addresses = fetch_params.iter().map(|fetch| fetch.address()).collect::<Vec<_>>();

		let (transfer_params, egress_ids) = batch_to_send
			.into_iter()
			.map(|ScheduledTransfer { egress_id, params }| (params, egress_id))
			.unzip::<_, _, Vec<_>, Vec<_>>();

		// Construct and send the transaction.
		match <T::ChainApiCall as AllBatch<T::TargetChain>>::new_unsigned(
			fetch_params,
			transfer_params,
		) {
			Ok(egress_transaction) => {
				let (broadcast_id, _) = T::Broadcaster::threshold_sign_and_broadcast_with_callback(
					egress_transaction,
					Call::finalise_ingress { addresses }.into(),
				);
				Self::deposit_event(Event::<T, I>::BatchBroadcastRequested {
					broadcast_id,
					egress_ids,
				});
				TransactionOutcome::Commit(Ok(()))
			},
			Err(AllBatchError::NotRequired) => TransactionOutcome::Commit(Ok(())),
			Err(AllBatchError::Other) => TransactionOutcome::Rollback(Err(DispatchError::Other(
				"AllBatch ApiCall creation failed, rolled back storage",
			))),
		}
	}

	/// Send all scheduled Cross Chain Messages out to the target chain.
	///
	/// Blacklisted assets are not sent and will remain in storage.
	fn do_egress_scheduled_ccm() {
		let ccms_to_send: Vec<CrossChainMessage<T::TargetChain>> =
			ScheduledEgressCcm::<T, I>::mutate(|ccms: &mut Vec<_>| {
				// Filter out disabled assets, and take up to batch_size requests to be sent.
				ccms.extract_if(|ccm| !DisabledEgressAssets::<T, I>::contains_key(ccm.asset()))
					.collect()
			});
		for ccm in ccms_to_send {
			match <T::ChainApiCall as ExecutexSwapAndCall<T::TargetChain>>::new_unsigned(
				ccm.egress_id,
				TransferAssetParams {
					asset: ccm.asset,
					amount: ccm.amount,
					to: ccm.destination_address,
				},
				ccm.source_chain,
				ccm.source_address,
				ccm.gas_budget,
				ccm.message.to_vec(),
			) {
				Ok(api_call) => {
					let (broadcast_id, _) = T::Broadcaster::threshold_sign_and_broadcast(api_call);
					Self::deposit_event(Event::<T, I>::CcmBroadcastRequested {
						broadcast_id,
						egress_id: ccm.egress_id,
					});
				},
				Err(error) => Self::deposit_event(Event::<T, I>::CcmEgressInvalid {
					egress_id: ccm.egress_id,
					error,
				}),
			};
		}
	}

	/// Completes a single deposit request.
	fn process_single_deposit(
		deposit_address: TargetChainAccount<T, I>,
		asset: TargetChainAsset<T, I>,
		amount: TargetChainAmount<T, I>,
		deposit_details: <T::TargetChain as Chain>::DepositDetails,
		block_height: TargetChainBlockNumber<T, I>,
	) -> DispatchResult {
		let deposit_channel_details = DepositChannelLookup::<T, I>::get(&deposit_address)
			.ok_or(Error::<T, I>::InvalidDepositAddress)?;

		if DepositChannelPool::<T, I>::get(deposit_channel_details.deposit_channel.channel_id)
			.is_some()
		{
			log_or_panic!(
				"Deposit channel {} should not be in the recycled address pool if it's active",
				deposit_channel_details.deposit_channel.channel_id
			);
			#[cfg(not(debug_assertions))]
			return Err(Error::<T, I>::InvalidDepositAddress.into())
		}

		ensure!(
			deposit_channel_details.deposit_channel.asset == asset,
			Error::<T, I>::AssetMismatch
		);

		if amount < MinimumDeposit::<T, I>::get(asset) {
			// If the amount is below the minimum allowed, the deposit is ignored.
			Self::deposit_event(Event::<T, I>::DepositIgnored {
				deposit_address,
				asset,
				amount,
				deposit_details,
			});
			return Ok(())
		}

		let channel_id = deposit_channel_details.deposit_channel.channel_id;
		Self::deposit_event(Event::<T, I>::DepositFetchesScheduled { channel_id, asset });

		match deposit_channel_details.action {
			ChannelAction::LiquidityProvision { lp_account, .. } =>
				T::LpBalance::try_credit_account(&lp_account, asset.into(), amount.into())?,
			ChannelAction::Swap {
				destination_address,
				destination_asset,
				broker_id,
				broker_commission_bps,
				..
			} => T::SwapDepositHandler::schedule_swap_from_channel(
				deposit_address.clone().into(),
				block_height.into(),
				asset.into(),
				destination_asset,
				amount.into(),
				destination_address,
				broker_id,
				broker_commission_bps,
				channel_id,
			),
			ChannelAction::CcmTransfer {
				destination_asset,
				destination_address,
				channel_metadata,
				..
			} => T::CcmHandler::on_ccm_deposit(
				asset.into(),
				amount.into(),
				destination_asset,
				destination_address,
				CcmDepositMetadata {
					source_chain: asset.into(),
					source_address: None,
					channel_metadata,
				},
				SwapOrigin::DepositChannel {
					deposit_address: T::AddressConverter::to_encoded_address(
						deposit_address.clone().into(),
					),
					channel_id,
					deposit_block_height: block_height.into(),
				},
			),
		};

		DepositBalances::<T, I>::mutate(asset, |deposits| {
			deposits.register_deposit(
				amount,
				&deposit_details,
				&deposit_channel_details.deposit_channel,
			)
		});

		Self::deposit_event(Event::DepositReceived {
			deposit_address,
			asset,
			amount,
			deposit_details,
		});
		Ok(())
	}

	fn expiry_and_recycle_block_height(
	) -> (TargetChainBlockNumber<T, I>, TargetChainBlockNumber<T, I>, TargetChainBlockNumber<T, I>)
	{
		let current_height = T::ChainTracking::get_block_height();
		let lifetime = DepositChannelLifetime::<T, I>::get();
		let expiry_height = current_height + lifetime;
		let recycle_height = expiry_height + lifetime;

		(current_height, expiry_height, recycle_height)
	}

	/// Opens a channel for the given asset and registers it with the given action.
	///
	/// May re-use an existing deposit address, depending on chain configuration.
	#[allow(clippy::type_complexity)]
	fn open_channel(
		source_asset: TargetChainAsset<T, I>,
		action: ChannelAction<T::AccountId>,
	) -> Result<(ChannelId, TargetChainAccount<T, I>, TargetChainBlockNumber<T, I>), DispatchError>
	{
		let (deposit_channel, channel_id) = if let Some((channel_id, mut deposit_channel)) =
			DepositChannelPool::<T, I>::drain_prefix(&Some(source_asset))
				.next()
				.or_else(|| DepositChannelPool::<T, I>::drain_prefix(None).next())
		{
			deposit_channel.asset = source_asset;
			(deposit_channel, channel_id)
		} else {
			let next_channel_id =
				ChannelIdCounter::<T, I>::try_mutate::<_, Error<T, I>, _>(|id| {
					*id = id.checked_add(1).ok_or(Error::<T, I>::ChannelIdsExhausted)?;
					Ok(*id)
				})?;
			(
				DepositChannel::generate_new::<T::AddressDerivation>(
					next_channel_id,
					source_asset,
				)?,
				next_channel_id,
			)
		};

		let deposit_address = deposit_channel.address.clone();

		let (current_height, expiry_height, recycle_height) =
			Self::expiry_and_recycle_block_height();

		DepositChannelRecycleBlocks::<T, I>::append((recycle_height, deposit_address.clone()));

		DepositChannelLookup::<T, I>::insert(
			&deposit_address,
			DepositChannelDetails {
				deposit_channel,
				opened_at: current_height,
				expires_at: expiry_height,
				action,
			},
		);

		Ok((channel_id, deposit_address, expiry_height))
	}
}

impl<T: Config<I>, I: 'static> EgressApi<T::TargetChain> for Pallet<T, I> {
	fn schedule_egress(
		asset: TargetChainAsset<T, I>,
		amount: TargetChainAmount<T, I>,
		destination_address: TargetChainAccount<T, I>,
		maybe_ccm_with_gas_budget: Option<(CcmDepositMetadata, TargetChainAmount<T, I>)>,
	) -> EgressId {
		let egress_counter = EgressIdCounter::<T, I>::mutate(|id| {
			*id = id.saturating_add(1);
			*id
		});
		let egress_id = (<T as Config<I>>::TargetChain::get(), egress_counter);
		match maybe_ccm_with_gas_budget {
			Some((
				CcmDepositMetadata { source_chain, source_address, channel_metadata },
				gas_budget,
			)) => ScheduledEgressCcm::<T, I>::append(CrossChainMessage {
				egress_id,
				asset,
				amount,
				destination_address: destination_address.clone(),
				message: channel_metadata.message,
				cf_parameters: channel_metadata.cf_parameters,
				source_chain,
				source_address,
				gas_budget,
			}),
			None => ScheduledTransfers::<T, I>::append(ScheduledTransfer {
				egress_id,
				params: TransferAssetParams { asset, amount, to: destination_address.clone() },
			}),
		}

		Self::deposit_event(Event::<T, I>::EgressScheduled {
			id: egress_id,
			asset,
			amount: amount.into(),
			destination_address,
		});

		egress_id
	}
}

impl<T: Config<I>, I: 'static> DepositApi<T::TargetChain> for Pallet<T, I> {
	type AccountId = T::AccountId;
	// This should be callable by the LP pallet.
	fn request_liquidity_deposit_address(
		lp_account: T::AccountId,
		source_asset: TargetChainAsset<T, I>,
	) -> Result<
		(ChannelId, ForeignChainAddress, <T::TargetChain as Chain>::ChainBlockNumber),
		DispatchError,
	> {
		let (channel_id, deposit_address, expiry_block) =
			Self::open_channel(source_asset, ChannelAction::LiquidityProvision { lp_account })?;

		Ok((channel_id, deposit_address.into(), expiry_block))
	}

	// This should only be callable by the broker.
	fn request_swap_deposit_address(
		source_asset: TargetChainAsset<T, I>,
		destination_asset: Asset,
		destination_address: ForeignChainAddress,
		broker_commission_bps: BasisPoints,
		broker_id: T::AccountId,
		channel_metadata: Option<CcmChannelMetadata>,
	) -> Result<
		(ChannelId, ForeignChainAddress, <T::TargetChain as Chain>::ChainBlockNumber),
		DispatchError,
	> {
		let (channel_id, deposit_address, expiry_height) = Self::open_channel(
			source_asset,
			match channel_metadata {
				Some(msg) => ChannelAction::CcmTransfer {
					destination_asset,
					destination_address,
					channel_metadata: msg,
				},
				None => ChannelAction::Swap {
					destination_asset,
					destination_address,
					broker_commission_bps,
					broker_id,
				},
			},
		)?;

		Ok((channel_id, deposit_address.into(), expiry_height))
	}
}
