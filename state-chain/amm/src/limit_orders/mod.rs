#[cfg(test)]
mod tests;

use std::collections::{btree_map, BTreeMap};

use primitive_types::{U256, U512};

use crate::common::{
	mul_div, mul_div_ceil, mul_div_floor, Amount, LiquidityProvider, OneToZero, Side, ZeroToOne,
	ONE_IN_PIPS, SqrtPriceQ64F96, is_sqrt_price_valid, MIN_SQRT_PRICE, MAX_SQRT_PRICE,
};

const MAX_FIXED_POOL_LIQUIDITY: Amount = U256([u64::MAX, u64::MAX, 0, 0]);
const PRICE_FRACTIONAL_BITS: u32 = 128;

type Price = U256;

#[derive(Clone, Debug, PartialEq, Eq)]
struct FloatBetweenZeroAndOne {
	normalised_mantissa: U256,
	negative_exponent: U256,
}
impl FloatBetweenZeroAndOne {
	fn max() -> Self {
		Self { normalised_mantissa: U256::max_value(), negative_exponent: U256::from(0) }
	}

	fn mul_div_ceil(&self, numerator: U256, denominator: U256) -> Self {
		assert!(!numerator.is_zero());
		assert!(numerator <= denominator);

		let (mul_normalised_mantissa, mul_normalise_shift) = {
			let unnormalised_mantissa = U256::full_mul(self.normalised_mantissa, numerator);
			let normalize_shift = unnormalised_mantissa.leading_zeros();
			(unnormalised_mantissa << normalize_shift, 256 - normalize_shift)
		};

		let (mul_div_normalised_mantissa, div_normalise_shift) = {
			let (d, div_remainder) =
				U512::div_mod(mul_normalised_mantissa, U512::from(denominator));
			let normalise_shift = d.leading_zeros();
			let (d, shift_remainder) = U512::div_mod(d, U512::one() << (256 - normalise_shift));
			let d = U256::try_from(d).unwrap();

			(
				if div_remainder.is_zero() && shift_remainder.is_zero() {
					d
				} else {
					d + U256::one()
				},
				normalise_shift,
			)
		};

		assert!(!mul_div_normalised_mantissa.is_zero());

		Self {
			normalised_mantissa: mul_div_normalised_mantissa,
			negative_exponent: self
				.negative_exponent
				.checked_add(U256::from(div_normalise_shift - mul_normalise_shift))
				.unwrap(),
		}
	}

	fn integer_mul_div(x: U256, numerator: &Self, denominator: &Self) -> (U256, U256) {
		let (shifted_y_floor, shifted_y_ceil) =
			mul_div(x, numerator.normalised_mantissa, denominator.normalised_mantissa);

		let negative_exponent =
			numerator.negative_exponent.checked_sub(denominator.negative_exponent).unwrap();

		(shifted_y_floor >> negative_exponent, {
			let y_ceil = shifted_y_ceil >> negative_exponent;
			if shifted_y_ceil != U256::zero() &&
				U256::from(shifted_y_ceil.trailing_zeros()) < negative_exponent
			{
				y_ceil + 1
			} else {
				y_ceil
			}
		})
	}
}

fn sqrt_price_to_price(sqrt_price: SqrtPriceQ64F96) -> Price {
	assert!((MIN_SQRT_PRICE..=MAX_SQRT_PRICE).contains(&sqrt_price));

	mul_div_floor(sqrt_price, sqrt_price, SqrtPriceQ64F96::one() << (2*96 - PRICE_FRACTIONAL_BITS))
}

pub trait SwapDirection: crate::common::SwapDirection {
	/// Calculates the swap input amount needed to produce an output amount at a price
	fn input_amount_ceil(output: Amount, price: Price) -> Amount;

	/// Calculates the swap input amount needed to produce an output amount at a price
	fn input_amount_floor(output: Amount, price: Price) -> Amount;

	/// Calculates the swap output amount produced for an input amount at a price
	fn output_amount_floor(input: Amount, price: Price) -> Amount;

	/// Gets entry for best prices pool
	fn best_priced_fixed_pool<'a>(
		pools: &'a mut BTreeMap<SqrtPriceQ64F96, FixedPool>,
	) -> Option<std::collections::btree_map::OccupiedEntry<'a, SqrtPriceQ64F96, FixedPool>>;
}
impl SwapDirection for ZeroToOne {
	fn input_amount_ceil(output: Amount, price: Price) -> Amount {
		mul_div_ceil(output, U256::one() << PRICE_FRACTIONAL_BITS, price)
	}

	fn input_amount_floor(output: Amount, price: Price) -> Amount {
		OneToZero::output_amount_floor(output, price)
	}

	fn output_amount_floor(input: Amount, price: Price) -> Amount {
		mul_div_floor(input, price, U256::one() << 128)
	}

	fn best_priced_fixed_pool<'a>(
		pools: &'a mut BTreeMap<SqrtPriceQ64F96, FixedPool>,
	) -> Option<std::collections::btree_map::OccupiedEntry<'a, SqrtPriceQ64F96, FixedPool>> {
		pools.last_entry()
	}
}
impl SwapDirection for OneToZero {
	fn input_amount_ceil(output: Amount, price: Price) -> Amount {
		mul_div_ceil(output, price, U256::one() << 128)
	}

	fn input_amount_floor(output: Amount, price: Price) -> Amount {
		ZeroToOne::output_amount_floor(output, price)
	}

	fn output_amount_floor(input: Amount, price: Price) -> Amount {
		mul_div_floor(input, U256::one() << 128, price)
	}

	fn best_priced_fixed_pool<'a>(
		pools: &'a mut BTreeMap<SqrtPriceQ64F96, FixedPool>,
	) -> Option<std::collections::btree_map::OccupiedEntry<'a, SqrtPriceQ64F96, FixedPool>> {
		pools.first_entry()
	}
}

#[derive(Debug)]
pub enum NewError {
	/// Fee must be between 0 - 50%
	InvalidFeeAmount,
}

#[derive(Debug)]
pub enum MintError {
	/// One of the start/end ticks of the range reached its maximum gross liquidity
	MaximumLiquidity,
}

#[derive(Debug)]
pub enum PositionError<T> {
	/// Invalid Price
	InvalidPrice,
	/// Position referenced does not exist
	NonExistent,
	Other(T),
}

#[derive(Debug)]
pub enum BurnError {
	/// Position referenced does not contain the requested liquidity
	PositionLacksLiquidity,
}

#[derive(Debug)]
pub enum CollectError {}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct CollectedAmounts {
	pub fees: Amount,
	pub swapped_liquidity: Amount,
}

#[derive(Clone, Debug)]
struct Position {
	pool_instance: u128,
	amount: Amount,
	percent_remaining: FloatBetweenZeroAndOne,
}

#[derive(Clone, Debug)]
pub struct FixedPool {
	pool_instance: u128,
	available: Amount,
	percent_remaining: FloatBetweenZeroAndOne,
}

#[derive(Clone, Debug)]
pub struct PoolState {
	fee_pips: u32,
	next_pool_instance: u128,
	fixed_pools: enum_map::EnumMap<Side, BTreeMap<SqrtPriceQ64F96, FixedPool>>,
	positions: enum_map::EnumMap<Side, BTreeMap<(SqrtPriceQ64F96, LiquidityProvider), Position>>,
}

impl PoolState {
	pub fn new(fee_pips: u32) -> Result<Self, NewError> {
		(fee_pips <= ONE_IN_PIPS / 2).then_some(()).ok_or(NewError::InvalidFeeAmount)?;

		Ok(Self {
			fee_pips,
			next_pool_instance: 0,
			fixed_pools: Default::default(),
			positions: Default::default(),
		})
	}

	pub fn current_sqrt_price<SD: SwapDirection>(&mut self) -> Option<SqrtPriceQ64F96> {
		SD::best_priced_fixed_pool(&mut self.fixed_pools[!SD::INPUT_SIDE]).map(|entry| *entry.key())
	}

	pub fn swap<SD: SwapDirection>(
		&mut self,
		mut amount: Amount,
		sqrt_price_limit: Option<SqrtPriceQ64F96>,
	) -> (Amount, Amount) {
		let mut total_output_amount = U256::zero();

		while let Some((sqrt_price, mut fixed_pool_entry)) = (amount != Amount::zero())
			.then_some(())
			.and_then(|()| SD::best_priced_fixed_pool(&mut self.fixed_pools[!SD::INPUT_SIDE]))
			.map(|entry| (*entry.key(), entry))
			.filter(|(sqrt_price, _)| {
				sqrt_price_limit.map_or(true, |sqrt_price_limit| {
					!SD::sqrt_price_op_more_than(*sqrt_price, sqrt_price_limit)
				})
			}) {
			let fixed_pool = fixed_pool_entry.get_mut();
				
			let amount_minus_fees = mul_div_floor(
				amount,
				U256::from(ONE_IN_PIPS - self.fee_pips),
				U256::from(ONE_IN_PIPS),
			); /* Will not overflow as fee_pips <= ONE_IN_PIPS / 2 */

			let price = sqrt_price_to_price(sqrt_price);
			let amount_required_to_consume_pool =
				SD::input_amount_ceil(fixed_pool.available, price);

			let output_amount = if amount_minus_fees >= amount_required_to_consume_pool {
				let fixed_pool = fixed_pool_entry.remove();

				// Cannot underflow as amount_minus_fees >= amount_required_to_consume_pool
				amount -= amount_required_to_consume_pool +
					mul_div_ceil(
						amount_required_to_consume_pool,
						U256::from(self.fee_pips),
						U256::from(ONE_IN_PIPS - self.fee_pips),
					); /* Will not overflow as fee_pips <= ONE_IN_PIPS / 2 */

				fixed_pool.available
			} else {
				let initial_output_amount = SD::output_amount_floor(amount_minus_fees, price);

				// We calculate (output_amount, next_percent_remaining) so that
				// next_percent_remaining is an under-estimate of the remaining liquidity, but also
				// an under-estimate of the used liquidity, by over-estimating it according to
				// used liquidity and then decreasing output_amount so that next_percent_remaining
				// also under-estimates the remainung_liquidity

				let next_percent_remaining = FloatBetweenZeroAndOne::mul_div_ceil(
					&fixed_pool.percent_remaining,
					/* Cannot underflow as amount_required_to_consume_pool is ceiled, but
					 * amount_minus_fees < amount_required_to_consume_pool, therefore
					 * amount_minus_fees <= SD::input_amount_floor(fixed_pool.available, price) */
					fixed_pool.available - initial_output_amount,
					fixed_pool.available,
				);

				// We back-calculate output_amount to ensure output_amount is less than or equal to
				// what percent_remaining suggests has been output
				let output_amount = fixed_pool.available -
					FloatBetweenZeroAndOne::integer_mul_div(
						fixed_pool.available,
						&next_percent_remaining,
						&fixed_pool.percent_remaining,
					)
					.1;

				assert!(output_amount <= initial_output_amount);

				fixed_pool.percent_remaining = next_percent_remaining;
				fixed_pool.available -= output_amount;
				amount = Amount::zero();

				output_amount
			};

			total_output_amount = total_output_amount.saturating_add(output_amount);
		}

		(total_output_amount, amount)
	}

	pub fn collect_and_mint<SD: SwapDirection>(
		&mut self,
		lp: LiquidityProvider,
		sqrt_price: SqrtPriceQ64F96,
		amount: Amount,
	) -> Result<CollectedAmounts, PositionError<MintError>> {
		if amount == Amount::zero() {
			self.inner_collect::<SD, _>(lp, sqrt_price)
		} else {
			Self::validate_sqrt_price(sqrt_price)?;
			let liquidity: Side = !SD::INPUT_SIDE;

			let mut next_pool_instance = self.next_pool_instance;
			let mut fixed_pool =
				self.fixed_pools[liquidity].get(&sqrt_price).cloned().unwrap_or_else(|| {
					next_pool_instance += 1;
					FixedPool {
						pool_instance: self.next_pool_instance,
						available: U256::zero(),
						percent_remaining: FloatBetweenZeroAndOne::max(),
					}
				});
			let (mut position, collected_amounts) = if let Some(mut position) =
				self.positions[liquidity].get(&(sqrt_price, lp)).cloned()
			{
				let (position, used_liquidity) =
					if position.pool_instance == fixed_pool.pool_instance {
						let (remaining_amount_floor, remaining_amount_ceil) =
							FloatBetweenZeroAndOne::integer_mul_div(
								position.amount,
								&fixed_pool.percent_remaining,
								&position.percent_remaining,
							);

						// We under-estimate used liquidity so that lp's don't receive more
						// swapped_liquidity and fees than may exist in the pool
						let used_liquidity = position.amount - remaining_amount_ceil;
						// We under-estimate remaining liquidity so that lp's cannot burn more liquidity
						// than truely exists in the pool
						position.amount = remaining_amount_floor;
						position.percent_remaining = fixed_pool.percent_remaining.clone();
						(position, used_liquidity)
					} else {
						(
							Position {
								pool_instance: fixed_pool.pool_instance,
								amount: U256::zero(),
								percent_remaining: fixed_pool.percent_remaining.clone(),
							},
							position.amount,
						)
					};

				(
					position,
					Self::collect_from_used_liquidity::<SD>(used_liquidity, sqrt_price_to_price(sqrt_price), self.fee_pips),
				)
			} else {
				(
					Position {
						pool_instance: fixed_pool.pool_instance,
						amount: U256::zero(),
						percent_remaining: fixed_pool.percent_remaining.clone(),
					},
					Default::default(),
				)
			};

			fixed_pool.available = fixed_pool.available.saturating_add(amount);
			if fixed_pool.available > MAX_FIXED_POOL_LIQUIDITY {
				Err(PositionError::Other(MintError::MaximumLiquidity))
			} else {
				position.amount += amount;

				self.next_pool_instance = next_pool_instance;
				self.fixed_pools[liquidity].insert(sqrt_price, fixed_pool);
				self.positions[liquidity].insert((sqrt_price, lp), position);

				Ok(collected_amounts)
			}
		}
	}

	fn validate_sqrt_price<T>(
		sqrt_price: SqrtPriceQ64F96,
	) -> Result<(), PositionError<T>> {
		is_sqrt_price_valid(sqrt_price)
			.then_some(())
			.ok_or(PositionError::InvalidPrice)
	}

	fn collect_from_used_liquidity<SD: SwapDirection>(
		used_liquidity: Amount,
		price: Price,
		fee_pips: u32,
	) -> CollectedAmounts {
		let swapped_liquidity = SD::input_amount_floor(used_liquidity, price);
		CollectedAmounts {
			fees: /* Will not overflow as fee_pips <= ONE_IN_PIPS / 2 */ mul_div_floor(
				swapped_liquidity,
				U256::from(fee_pips),
				U256::from(ONE_IN_PIPS - fee_pips),
			),
			swapped_liquidity,
		}
	}

	pub fn collect_and_burn<SD: SwapDirection>(
		&mut self,
		lp: LiquidityProvider,
		sqrt_price: SqrtPriceQ64F96,
		amount: Amount,
	) -> Result<(Amount, CollectedAmounts), PositionError<BurnError>> {
		Self::validate_sqrt_price(sqrt_price)?;
		let mut position_entry = match self.positions[!SD::INPUT_SIDE].entry((sqrt_price, lp)) {
			btree_map::Entry::Occupied(entry) => Some(entry),
			_ => None,
		}
		.ok_or(PositionError::NonExistent)?;
		let position = position_entry.get_mut();
		let price = sqrt_price_to_price(sqrt_price);

		if let Some(mut fixed_pool_entry) = match self.fixed_pools[!SD::INPUT_SIDE].entry(sqrt_price) {
			btree_map::Entry::Occupied(entry)
				if entry.get().pool_instance == position.pool_instance =>
				Some(entry),
			_ => None,
		} {
			let fixed_pool = fixed_pool_entry.get_mut();

			let (remaining_amount_floor, remaining_amount_ceil) =
				FloatBetweenZeroAndOne::integer_mul_div(
					position.amount,
					&fixed_pool.percent_remaining,
					&position.percent_remaining,
				);
			let collected_amounts = Self::collect_from_used_liquidity::<SD>(
				position.amount - remaining_amount_ceil,
				price,
				self.fee_pips,
			);
			position.percent_remaining = fixed_pool.percent_remaining.clone();
			position.amount = remaining_amount_floor
				.checked_sub(amount)
				.ok_or(PositionError::Other(BurnError::PositionLacksLiquidity))?;
			fixed_pool.available = fixed_pool.available.checked_sub(amount).unwrap(); // This doesn't underflow as remaining_amount is an underestimate as
																		  // fixed_pool.percent_remaining is rounded down

			if position.amount == Amount::zero() {
				position_entry.remove();

				if fixed_pool.available == Amount::zero() {
					fixed_pool_entry.remove();
				}
			}

			Ok((amount, collected_amounts))
		} else if amount == Amount::zero() {
			let position = position_entry.remove();
			Ok((
				Amount::zero(),
				Self::collect_from_used_liquidity::<SD>(position.amount, price, self.fee_pips),
			))
		} else {
			Err(PositionError::Other(BurnError::PositionLacksLiquidity))
		}
	}

	pub fn collect<SD: SwapDirection>(
		&mut self,
		lp: LiquidityProvider,
		sqrt_price: SqrtPriceQ64F96,
	) -> Result<CollectedAmounts, PositionError<CollectError>> {
		self.inner_collect::<SD, _>(lp, sqrt_price)
	}

	fn inner_collect<SD: SwapDirection, E>(
		&mut self,
		lp: LiquidityProvider,
		sqrt_price: SqrtPriceQ64F96,
	) -> Result<CollectedAmounts, PositionError<E>> {
		Self::validate_sqrt_price(sqrt_price)?;
		let mut position_entry = match self.positions[!SD::INPUT_SIDE].entry((sqrt_price, lp)) {
			btree_map::Entry::Occupied(entry) => Some(entry),
			_ => None,
		}
		.ok_or(PositionError::NonExistent)?;
		let position = position_entry.get_mut();
		let price = sqrt_price_to_price(sqrt_price);

		Ok(
			if let Some(mut fixed_pool_entry) = match self.fixed_pools[!SD::INPUT_SIDE].entry(sqrt_price)
			{
				btree_map::Entry::Occupied(entry)
					if entry.get().pool_instance == position.pool_instance =>
					Some(entry),
				_ => None,
			} {
				let fixed_pool = fixed_pool_entry.get_mut();

				let (remaining_amount_floor, remaining_amount_ceil) =
					FloatBetweenZeroAndOne::integer_mul_div(
						position.amount,
						&fixed_pool.percent_remaining,
						&position.percent_remaining,
					);
				let collected_amounts = Self::collect_from_used_liquidity::<SD>(
					position.amount - remaining_amount_ceil,
					price,
					self.fee_pips,
				);
				position.percent_remaining = fixed_pool.percent_remaining.clone();
				position.amount = remaining_amount_floor;

				if position.amount == Amount::zero() {
					position_entry.remove();
				}

				collected_amounts
			} else {
				let position = position_entry.remove();
				Self::collect_from_used_liquidity::<SD>(position.amount, price, self.fee_pips)
			},
		)
	}
}
