#![cfg_attr(not(feature = "std"), no_std)]
use core::ops::Range;

use cf_amm::{
	common::{Order, Price, Side},
	range_orders, NewError, PoolState,
};
use cf_primitives::{chains::assets::any, Asset, AssetAmount, SwapLeg, SwapOutput, STABLE_ASSET};
use cf_traits::{impl_pallet_safe_mode, Chainflip, SwappingApi};
use frame_support::{
	pallet_prelude::*,
	sp_runtime::{Permill, Saturating},
	transactional,
};
use frame_system::pallet_prelude::OriginFor;
use serde::{Deserialize, Serialize};
use sp_arithmetic::traits::Zero;

pub use pallet::*;

mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

impl_pallet_safe_mode!(PalletSafeMode; minting_range_order_enabled, minting_limit_order_enabled, burning_range_order_enabled, burning_limit_order_enabled);

enum Stability {
	Stable,
	Unstable,
}

struct CanonialAssetPair<T: Config> {
	assets: cf_amm::common::SideMap<Asset>,
	_phantom: core::marker::PhantomData<T>,
}
impl<T: Config> CanonialAssetPair<T> {
	fn new(base_asset: Asset, pair_asset: Asset) -> Result<Self, Error<T>> {
		match (base_asset, pair_asset) {
			(STABLE_ASSET, STABLE_ASSET) => Err(Error::<T>::PoolDoesNotExist),
			(STABLE_ASSET, unstable_asset) | (unstable_asset, STABLE_ASSET) => Ok(Self {
				assets: cf_amm::common::SideMap::<()>::default().map(|side, _| {
					match Self::side_to_stability(side) {
						Stability::Stable => STABLE_ASSET,
						Stability::Unstable => unstable_asset,
					}
				}),
				_phantom: Default::default(),
			}),
			_ => Err(Error::<T>::PoolDoesNotExist),
		}
	}

	fn side_to_asset(&self, side: Side) -> Asset {
		self.assets[side]
	}

	/// !!! Must match side_to_stability !!!
	fn stability_to_side(stability: Stability) -> Side {
		match stability {
			Stability::Stable => Side::One,
			Stability::Unstable => Side::Zero,
		}
	}

	/// !!! Must match stability_to_side !!!
	fn side_to_stability(side: Side) -> Stability {
		match side {
			Side::Zero => Stability::Unstable,
			Side::One => Stability::Stable,
		}
	}
}

struct AssetPair<T: Config> {
	canonial_asset_pair: CanonialAssetPair<T>,
	base_side: Side,
}
impl<T: Config> AssetPair<T> {
	fn new(base_asset: Asset, pair_asset: Asset) -> Result<Self, Error<T>> {
		Ok(Self {
			canonial_asset_pair: CanonialAssetPair::new(base_asset, pair_asset)?,
			base_side: CanonialAssetPair::<T>::stability_to_side(match (base_asset, pair_asset) {
				(STABLE_ASSET, STABLE_ASSET) => Err(Error::<T>::PoolDoesNotExist),
				(STABLE_ASSET, _unstable_asset) => Ok(Stability::Stable),
				(_unstable_asset, STABLE_ASSET) => Ok(Stability::Unstable),
				_ => Err(Error::<T>::PoolDoesNotExist),
			}?),
		})
	}

	fn asset_amounts_to_side_map(
		&self,
		asset_amounts: AssetAmounts,
	) -> cf_amm::common::SideMap<cf_amm::common::Amount> {
		cf_amm::common::SideMap::from_array(match self.base_side {
			Side::Zero => [asset_amounts.base.into(), asset_amounts.pair.into()],
			Side::One => [asset_amounts.pair.into(), asset_amounts.base.into()],
		})
	}

	fn side_map_to_asset_amounts(
		&self,
		side_map: cf_amm::common::SideMap<cf_amm::common::Amount>,
	) -> Result<AssetAmounts, <cf_amm::common::Amount as TryInto<AssetAmount>>::Error> {
		let side_map = side_map.try_map(|_, amount| amount.try_into())?;
		Ok(AssetAmounts { base: side_map[self.base_side], pair: side_map[!self.base_side] })
	}

	fn try_debit_asset_amounts(
		&self,
		lp: &T::AccountId,
		AssetAmounts { base, pair }: AssetAmounts,
	) -> DispatchResult {
		use cf_traits::LpBalanceApi;

		T::LpBalance::try_debit_account(
			lp,
			self.canonial_asset_pair.side_to_asset(self.base_side),
			base,
		)?;
		T::LpBalance::try_debit_account(
			lp,
			self.canonial_asset_pair.side_to_asset(!self.base_side),
			pair,
		)?;
		Ok(())
	}

	fn try_credit_asset_amounts(
		&self,
		lp: &T::AccountId,
		AssetAmounts { base, pair }: AssetAmounts,
	) -> DispatchResult {
		use cf_traits::LpBalanceApi;

		T::LpBalance::try_credit_account(
			lp,
			self.canonial_asset_pair.side_to_asset(self.base_side),
			base,
		)?;
		T::LpBalance::try_credit_account(
			lp,
			self.canonial_asset_pair.side_to_asset(!self.base_side),
			pair,
		)?;
		Ok(())
	}
}

#[frame_support::pallet]
pub mod pallet {
	use cf_amm::{
		common::Tick,
		limit_orders,
		range_orders::{self, Liquidity},
	};
	use cf_traits::{AccountRoleRegistry, LpBalanceApi};
	use frame_system::pallet_prelude::BlockNumberFor;

	use super::*;

	#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
	pub struct Pool<LiquidityProvider> {
		pub enabled: bool,
		pub pool_state: PoolState<LiquidityProvider>,
	}

	pub type OrderId = u64;

	#[derive(
		Copy,
		Clone,
		Debug,
		Encode,
		Decode,
		TypeInfo,
		MaxEncodedLen,
		PartialEq,
		Eq,
		Deserialize,
		Serialize,
	)]
	pub struct AssetAmounts {
		pub base: AssetAmount,
		pub pair: AssetAmount,
	}

	#[derive(
		Copy,
		Clone,
		Debug,
		Encode,
		Decode,
		TypeInfo,
		MaxEncodedLen,
		PartialEq,
		Eq,
		Deserialize,
		Serialize,
	)]
	pub enum RangeOrderSize {
		AssetAmounts { maximum: AssetAmounts, minimum: AssetAmounts },
		Liquidity { liquidity: Liquidity },
	}

	#[derive(
		Copy,
		Clone,
		Debug,
		Encode,
		Decode,
		TypeInfo,
		MaxEncodedLen,
		PartialEq,
		Eq,
		Deserialize,
		Serialize,
	)]
	pub enum IncreaseOrDecrease {
		Increase,
		Decrease,
	}

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: Chainflip {
		/// The event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Pallet responsible for managing Liquidity Providers.
		type LpBalance: LpBalanceApi<AccountId = Self::AccountId>;

		#[pallet::constant]
		type NetworkFee: Get<Permill>;

		/// Safe Mode access.
		type SafeMode: Get<PalletSafeMode>;

		/// Benchmark weights
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Pools are indexed by single asset since USDC is implicit.
	/// The STABLE_ASSET is always PoolSide::Asset1
	#[pallet::storage]
	pub type Pools<T: Config> =
		StorageMap<_, Twox64Concat, any::Asset, Pool<T::AccountId>, OptionQuery>;

	/// FLIP ready to be burned.
	#[pallet::storage]
	pub(super) type FlipToBurn<T: Config> = StorageValue<_, AssetAmount, ValueQuery>;

	/// Interval at which we buy FLIP in order to burn it.
	#[pallet::storage]
	pub(super) type FlipBuyInterval<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Network fees, in USDC terms, that have been collected and are ready to be converted to FLIP.
	#[pallet::storage]
	pub type CollectedNetworkFee<T: Config> = StorageValue<_, AssetAmount, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub flip_buy_interval: BlockNumberFor<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			FlipBuyInterval::<T>::set(self.flip_buy_interval);
		}
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { flip_buy_interval: BlockNumberFor::<T>::zero() }
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(current_block: BlockNumberFor<T>) -> Weight {
			let mut weight_used: Weight = T::DbWeight::get().reads(1);
			let interval = FlipBuyInterval::<T>::get();
			if interval.is_zero() {
				log::debug!("Flip buy interval is zero, skipping.")
			} else {
				weight_used.saturating_accrue(T::DbWeight::get().reads(1));
				if (current_block % interval).is_zero() &&
					!CollectedNetworkFee::<T>::get().is_zero()
				{
					weight_used.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
					if let Err(e) = CollectedNetworkFee::<T>::try_mutate(|collected_fee| {
						let flip_to_burn = Self::swap_single_leg(
							SwapLeg::FromStable,
							any::Asset::Flip,
							*collected_fee,
						)?;
						FlipToBurn::<T>::mutate(|total| {
							total.saturating_accrue(flip_to_burn);
						});
						collected_fee.set_zero();
						Ok::<_, DispatchError>(())
					}) {
						log::warn!("Unable to swap Network Fee to Flip: {e:?}");
					}
				}
			}
			weight_used
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Setting the buy interval to zero is not allowed.
		ZeroBuyIntervalNotAllowed,
		/// The specified exchange pool already exists.
		PoolAlreadyExists,
		/// The specified exchange pool does not exist.
		PoolDoesNotExist,
		/// The exchange pool is currently disabled.
		PoolDisabled,
		/// the Fee BIPs must be within the allowed range.
		InvalidFeeAmount,
		/// the initial price must be within the allowed range.
		InvalidInitialPrice,
		/// The Upper or Lower tick is invalid.
		InvalidTickRange,
		/// The tick is invalid.
		InvalidTick,
		/// One of the referenced ticks reached its maximum gross liquidity
		MaximumGrossLiquidity,
		/// The user's position does not exist.
		PositionDoesNotExist,
		/// It is no longer possible to mint limit orders due to reaching the maximum pool
		/// instances, other than for ticks where a fixed pool currently exists.
		MaximumPoolInstances,
		/// The pool does not have enough liquidity left to process the swap.
		InsufficientLiquidity,
		/// The swap output is past the maximum allowed amount.
		OutputOverflow,
		/// There are no amounts between the specified maximum and minimum that match the required
		/// ratio of assets
		AssetRatioUnachieveable,
		/// Minting Range Order is disabled
		MintingRangeOrderDisabled,
		/// Burning Range Order is disabled
		BurningRangeOrderDisabled,
		/// Minting Limit Order is disabled
		MintingLimitOrderDisabled,
		/// Burning Limit Order is disabled
		BurningLimitOrderDisabled,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		UpdatedBuyInterval {
			buy_interval: BlockNumberFor<T>,
		},
		PoolStateUpdated {
			unstable_asset: any::Asset,
			enabled: bool,
		},
		NewPoolCreated {
			unstable_asset: any::Asset,
			fee_hundredth_pips: u32,
			initial_price: Price,
		},
		RangeOrderUpdated {
			lp: T::AccountId,
			base_asset: Asset,
			pair_asset: Asset,
			id: OrderId,
			tick_range: core::ops::Range<Tick>,
			increase_or_decrease: IncreaseOrDecrease,
			liquidity_delta: Liquidity,
			liquidity_total: Liquidity,
			assets_delta: AssetAmounts,
			collected_fees: AssetAmounts,
		},
		LimitOrderUpdated {
			lp: T::AccountId,
			sell_asset: any::Asset,
			buy_asset: any::Asset,
			id: OrderId,
			tick: Tick,
			increase_or_decrease: IncreaseOrDecrease,
			amount_delta: AssetAmount,
			amount_total: AssetAmount,
			collected_fees: AssetAmount,
			swapped_liquidity: AssetAmount,
		},
		NetworkFeeTaken {
			fee_amount: AssetAmount,
		},
		AssetSwapped {
			from: any::Asset,
			to: any::Asset,
			input_amount: AssetAmount,
			output_amount: AssetAmount,
		},
		LiquidityFeeUpdated {
			unstable_asset: any::Asset,
			fee_hundredth_pips: u32,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Updates the buy interval.
		///
		/// ## Events
		///
		/// - [UpdatedBuyInterval](Event::UpdatedBuyInterval)
		///
		/// ## Errors
		///
		/// - [BadOrigin](frame_system::BadOrigin)
		/// - [ZeroBuyIntervalNotAllowed](pallet_cf_pools::Error::ZeroBuyIntervalNotAllowed)
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::update_buy_interval())]
		pub fn update_buy_interval(
			origin: OriginFor<T>,
			new_buy_interval: BlockNumberFor<T>,
		) -> DispatchResult {
			T::EnsureGovernance::ensure_origin(origin)?;
			ensure!(new_buy_interval != Zero::zero(), Error::<T>::ZeroBuyIntervalNotAllowed);
			FlipBuyInterval::<T>::set(new_buy_interval);
			Self::deposit_event(Event::<T>::UpdatedBuyInterval { buy_interval: new_buy_interval });
			Ok(())
		}

		/// Enable or disable an exchange pool.
		/// Requires Governance.
		///
		/// ## Events
		///
		/// - [On update](Event::PoolStateUpdated)
		///
		/// ## Errors
		///
		/// - [BadOrigin](frame_system::BadOrigin)
		/// - [PoolDoesNotExist](pallet_cf_pools::Error::PoolDoesNotExist)
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::update_pool_enabled())]
		pub fn update_pool_enabled(
			origin: OriginFor<T>,
			unstable_asset: any::Asset,
			enabled: bool,
		) -> DispatchResult {
			T::EnsureGovernance::ensure_origin(origin)?;
			Pools::<T>::try_mutate(unstable_asset, |maybe_pool| {
				let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;
				pool.enabled = enabled;
				Self::deposit_event(Event::<T>::PoolStateUpdated { unstable_asset, enabled });
				Ok(())
			})
		}

		/// Create a new pool. Pools are enabled by default.
		/// Requires Governance.
		///
		/// ## Events
		///
		/// - [On success](Event::NewPoolCreated)
		///
		/// ## Errors
		///
		/// - [BadOrigin](frame_system::BadOrigin)
		/// - [InvalidFeeAmount](pallet_cf_pools::Error::InvalidFeeAmount)
		/// - [InvalidTick](pallet_cf_pools::Error::InvalidTick)
		/// - [InvalidInitialPrice](pallet_cf_pools::Error::InvalidInitialPrice)
		/// - [PoolAlreadyExists](pallet_cf_pools::Error::PoolAlreadyExists)
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::new_pool())]
		pub fn new_pool(
			origin: OriginFor<T>,
			unstable_asset: any::Asset,
			fee_hundredth_pips: u32,
			initial_price: Price,
		) -> DispatchResult {
			T::EnsureGovernance::ensure_origin(origin)?;
			Pools::<T>::try_mutate(unstable_asset, |maybe_pool| {
				ensure!(maybe_pool.is_none(), Error::<T>::PoolAlreadyExists);

				*maybe_pool = Some(Pool {
					enabled: true,
					pool_state: PoolState::new(fee_hundredth_pips, initial_price).map_err(|e| {
						match e {
							NewError::LimitOrders(limit_orders::NewError::InvalidFeeAmount) =>
								Error::<T>::InvalidFeeAmount,
							NewError::RangeOrders(range_orders::NewError::InvalidFeeAmount) =>
								Error::<T>::InvalidFeeAmount,
							NewError::RangeOrders(range_orders::NewError::InvalidInitialPrice) =>
								Error::<T>::InvalidInitialPrice,
						}
					})?,
				});

				Ok::<_, Error<T>>(())
			})?;

			Self::deposit_event(Event::<T>::NewPoolCreated {
				unstable_asset,
				fee_hundredth_pips,
				initial_price,
			});

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::zero())]
		pub fn update_range_order(
			origin: OriginFor<T>,
			base_asset: Asset,
			pair_asset: Asset,
			id: OrderId,
			tick_range: Option<core::ops::Range<Tick>>,
			size: RangeOrderSize,
			increase_or_decrease: IncreaseOrDecrease, // TODO Change order
		) -> DispatchResult {
			let lp = T::AccountRoleRegistry::ensure_liquidity_provider(origin)?;

			let asset_pair = AssetPair::<T>::new(base_asset, pair_asset)?;
			Pools::<T>::try_mutate(
				asset_pair
					.canonial_asset_pair
					.side_to_asset(CanonialAssetPair::<T>::stability_to_side(Stability::Unstable)),
				|maybe_pool| {
					let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;
					ensure!(pool.enabled, Error::<T>::PoolDisabled);

					let size = match size {
						RangeOrderSize::Liquidity { liquidity } =>
							range_orders::Size::Liquidity { liquidity },
						RangeOrderSize::AssetAmounts { maximum, minimum } =>
							range_orders::Size::Amount {
								maximum: asset_pair.asset_amounts_to_side_map(maximum),
								minimum: asset_pair.asset_amounts_to_side_map(minimum),
							},
					};

					Self::inner_update_range_order(
						pool,
						&lp,
						&asset_pair,
						id,
						tick_range.unwrap(),
						size,
						increase_or_decrease,
					)
				},
			)
		}

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::zero())]
		pub fn set_range_order(
			origin: OriginFor<T>,
			base_asset: Asset,
			pair_asset: Asset,
			id: OrderId,
			tick_range: Option<core::ops::Range<Tick>>,
			size: RangeOrderSize,
		) -> DispatchResult {
			let lp = T::AccountRoleRegistry::ensure_liquidity_provider(origin)?;

			let asset_pair = AssetPair::<T>::new(base_asset, pair_asset)?;
			Pools::<T>::try_mutate(
				asset_pair
					.canonial_asset_pair
					.side_to_asset(CanonialAssetPair::<T>::stability_to_side(Stability::Unstable)),
				|maybe_pool| {
					let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;
					ensure!(pool.enabled, Error::<T>::PoolDisabled);

					let size = match size {
						RangeOrderSize::Liquidity { liquidity } =>
							range_orders::Size::Liquidity { liquidity },
						RangeOrderSize::AssetAmounts { maximum, minimum } =>
							range_orders::Size::Amount {
								maximum: asset_pair.asset_amounts_to_side_map(maximum),
								minimum: asset_pair.asset_amounts_to_side_map(minimum),
							},
					};

					Self::inner_update_range_order(
						pool,
						&lp,
						&asset_pair,
						id,
						tick_range.clone().unwrap(),
						range_orders::Size::Liquidity { liquidity: Liquidity::MAX },
						IncreaseOrDecrease::Decrease,
					)?;
					Self::inner_update_range_order(
						pool,
						&lp,
						&asset_pair,
						id,
						tick_range.unwrap(),
						size,
						IncreaseOrDecrease::Increase,
					)?;

					Ok(())
				},
			)
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::zero())]
		pub fn update_limit_order(
			_origin: OriginFor<T>,
			_sell_asset: any::Asset,
			_buy_asset: any::Asset,
			_id: OrderId,
			_tick: Option<Tick>,
			_sell_amount: AssetAmount,
			_increase_or_decrease: IncreaseOrDecrease,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(Weight::zero())]
		pub fn set_limit_order(
			_origin: OriginFor<T>,
			_sell_asset: any::Asset,
			_buy_asset: any::Asset,
			_id: OrderId,
			_tick: Option<Tick>,
			_sell_amount: AssetAmount,
		) -> DispatchResult {
			Ok(())
		}
	}
}

impl<T: Config> SwappingApi for Pallet<T> {
	fn take_network_fee(input: AssetAmount) -> AssetAmount {
		if input.is_zero() {
			return input
		}
		let (remaining, fee) = utilities::calculate_network_fee(T::NetworkFee::get(), input);
		CollectedNetworkFee::<T>::mutate(|total| {
			total.saturating_accrue(fee);
		});
		Self::deposit_event(Event::<T>::NetworkFeeTaken { fee_amount: fee });
		remaining
	}

	#[transactional]
	fn swap_single_leg(
		leg: SwapLeg,
		unstable_asset: any::Asset,
		input_amount: AssetAmount,
	) -> Result<AssetAmount, DispatchError> {
		Self::try_mutate_pool_state(unstable_asset, |pool_state| {
			let (from, to, output_amount) = match leg {
				SwapLeg::FromStable => (STABLE_ASSET, unstable_asset, {
					let (output_amount, remaining_amount) =
						pool_state.swap(Side::One, Order::Sell, input_amount.into());
					remaining_amount
						.is_zero()
						.then_some(())
						.ok_or(Error::<T>::InsufficientLiquidity)?;
					output_amount
				}),
				SwapLeg::ToStable => (unstable_asset, STABLE_ASSET, {
					let (output_amount, remaining_amount) =
						pool_state.swap(Side::Zero, Order::Sell, input_amount.into());
					remaining_amount
						.is_zero()
						.then_some(())
						.ok_or(Error::<T>::InsufficientLiquidity)?;
					output_amount
				}),
			};
			let output_amount = output_amount.try_into().map_err(|_| Error::<T>::OutputOverflow)?;
			Self::deposit_event(Event::<T>::AssetSwapped { from, to, input_amount, output_amount });
			Ok(output_amount)
		})
	}
}

impl<T: Config> cf_traits::FlipBurnInfo for Pallet<T> {
	fn take_flip_to_burn() -> AssetAmount {
		FlipToBurn::<T>::take()
	}
}

impl<T: Config> Pallet<T> {
	fn inner_update_range_order(
		pool: &mut Pool<T::AccountId>,
		lp: &T::AccountId,
		asset_pair: &AssetPair<T>,
		id: OrderId,
		tick_range: Range<cf_amm::common::Tick>,
		size: range_orders::Size,
		increase_or_decrease: IncreaseOrDecrease,
	) -> DispatchResult {
		let (liquidity_delta, liquidity_total, assets_delta, collected_fees) =
			match increase_or_decrease {
				IncreaseOrDecrease::Increase => {
					let (assets_debited, minted_liquidity, collected, position_info) = pool
						.pool_state
						.collect_and_mint_range_order(
							lp,
							tick_range.clone(),
							size,
							|required_amounts| {
								let required_amounts =
									asset_pair.side_map_to_asset_amounts(required_amounts)?;
								asset_pair
									.try_debit_asset_amounts(lp, required_amounts)
									.map(|()| required_amounts)
							},
						)
						.map_err(|e| match e {
							range_orders::PositionError::InvalidTickRange =>
								Error::<T>::InvalidTickRange.into(),
							range_orders::PositionError::NonExistent =>
								Error::<T>::PositionDoesNotExist.into(),
							range_orders::PositionError::Other(
								range_orders::MintError::CallbackFailed(e),
							) => e,
							range_orders::PositionError::Other(
								range_orders::MintError::MaximumGrossLiquidity,
							) => Error::<T>::MaximumGrossLiquidity.into(),
							range_orders::PositionError::Other(
								cf_amm::range_orders::MintError::AssetRatioUnachieveable,
							) => Error::<T>::AssetRatioUnachieveable.into(),
						})?;

					let collected_fees = asset_pair.side_map_to_asset_amounts(collected.fees)?;
					asset_pair.try_credit_asset_amounts(lp, collected_fees)?;

					(minted_liquidity, position_info.liquidity, assets_debited, collected_fees)
				},
				IncreaseOrDecrease::Decrease => {
					let (assets_withdrawn, burnt_liquidity, collected, position_info) = pool
						.pool_state
						.collect_and_burn_range_order(lp, tick_range.clone(), size)
						.map_err(|e| match e {
							range_orders::PositionError::InvalidTickRange =>
								Error::<T>::InvalidTickRange,
							range_orders::PositionError::NonExistent =>
								Error::<T>::PositionDoesNotExist,
							range_orders::PositionError::Other(e) => match e {
								range_orders::BurnError::AssetRatioUnachieveable =>
									Error::<T>::AssetRatioUnachieveable,
							},
						})?;

					let assets_withdrawn =
						asset_pair.side_map_to_asset_amounts(assets_withdrawn)?;
					asset_pair.try_credit_asset_amounts(lp, assets_withdrawn)?;

					let collected_fees = asset_pair.side_map_to_asset_amounts(collected.fees)?;
					asset_pair.try_credit_asset_amounts(lp, collected_fees)?;

					(burnt_liquidity, position_info.liquidity, assets_withdrawn, collected_fees)
				},
			};

		Self::deposit_event(Event::<T>::RangeOrderUpdated {
			lp: lp.clone(),
			base_asset: asset_pair.canonial_asset_pair.side_to_asset(asset_pair.base_side),
			pair_asset: asset_pair.canonial_asset_pair.side_to_asset(!asset_pair.base_side),
			id,
			tick_range,
			increase_or_decrease,
			liquidity_delta,
			liquidity_total,
			assets_delta,
			collected_fees,
		});

		Ok(())
	}

	#[transactional]
	pub fn swap_with_network_fee(
		from: any::Asset,
		to: any::Asset,
		input_amount: AssetAmount,
	) -> Result<SwapOutput, DispatchError> {
		Ok(match (from, to) {
			(input_asset, STABLE_ASSET) => Self::take_network_fee(Self::swap_single_leg(
				SwapLeg::ToStable,
				input_asset,
				input_amount,
			)?)
			.into(),
			(STABLE_ASSET, output_asset) => Self::swap_single_leg(
				SwapLeg::FromStable,
				output_asset,
				Self::take_network_fee(input_amount),
			)?
			.into(),
			(input_asset, output_asset) => {
				let intermediate_output =
					Self::swap_single_leg(SwapLeg::ToStable, input_asset, input_amount)?;
				SwapOutput {
					intermediary: Some(intermediate_output),
					output: Self::swap_single_leg(
						SwapLeg::FromStable,
						output_asset,
						Self::take_network_fee(intermediate_output),
					)?,
				}
			},
		})
	}

	pub fn get_pool(asset: Asset) -> Option<Pool<T::AccountId>> {
		Pools::<T>::get(asset)
	}

	fn try_mutate_pool_state<
		R,
		E: From<pallet::Error<T>>,
		F: FnOnce(&mut PoolState<T::AccountId>) -> Result<R, E>,
	>(
		asset: any::Asset,
		f: F,
	) -> Result<R, E> {
		Pools::<T>::try_mutate(asset, |maybe_pool| {
			let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;
			ensure!(pool.enabled, Error::<T>::PoolDisabled);
			f(&mut pool.pool_state)
		})
	}

	pub fn current_price(from: Asset, to: Asset) -> Option<Price> {
		match (from, to) {
			(STABLE_ASSET, unstable_asset) => Pools::<T>::get(unstable_asset)
				.and_then(|mut pool| pool.pool_state.current_price(Side::One, Order::Sell)),
			(unstable_asset, STABLE_ASSET) => Pools::<T>::get(unstable_asset)
				.and_then(|mut pool| pool.pool_state.current_price(Side::One, Order::Buy)),
			_ => None,
		}
	}
}

pub mod utilities {
	use super::*;

	pub fn side_to_asset(unstable_asset: Asset, side: Side) -> Asset {
		match side {
			Side::Zero => unstable_asset,
			Side::One => STABLE_ASSET,
		}
	}

	pub fn order_to_side(order: Order) -> Side {
		match order {
			Order::Buy => Side::One,
			Order::Sell => Side::Zero,
		}
	}

	pub fn calculate_network_fee(
		fee_percentage: Permill,
		input: AssetAmount,
	) -> (AssetAmount, AssetAmount) {
		let fee = fee_percentage * input;
		(input - fee, fee)
	}
}
