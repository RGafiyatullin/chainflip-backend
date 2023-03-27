use crate::{common::{sqrt_price_at_tick, MIN_TICK, MAX_TICK}, limit_orders, range_orders};

use super::*;

use cf_utilities::{assert_panics, assert_ok};
use rand::{SeedableRng, prelude::Distribution, Rng};

/// The amounts used as parameters to input_amount_floor, input_amount_ceil, output_amount_floor are guaranteed to be <= MAX_FIXED_POOL_LIQUIDITY.
/// This test checks that MAX_FIXED_POOL_LIQUIDITY is set low enough that those calculations don't overflow.
#[test]
fn max_liquidity() {
	macro_rules! checks {
		($t:ty, $price:ident) => {
			<$t>::input_amount_floor(MAX_FIXED_POOL_LIQUIDITY, $price);
			<$t>::input_amount_ceil(MAX_FIXED_POOL_LIQUIDITY, $price);
			<$t>::output_amount_floor(MAX_FIXED_POOL_LIQUIDITY, $price);	
		}
	}

	for price in [MIN_SQRT_PRICE, MAX_SQRT_PRICE].map(sqrt_price_to_price) {
		checks!(ZeroToOne, price);
		checks!(OneToZero, price);
	}
}

#[test]
fn test_sqrt_price_to_price() {
	assert_eq!(sqrt_price_to_price(SqrtPriceQ64F96::from(1) << 96), Price::from(1) << PRICE_FRACTIONAL_BITS);
	assert!(sqrt_price_to_price(MIN_SQRT_PRICE) < sqrt_price_to_price(MAX_SQRT_PRICE));
}

#[test]
fn test_float() {
	let mut rng = rand::rngs::StdRng::from_seed([8u8; 32]);

	fn rng_u256(rng: &mut impl rand::Rng) -> U256 {
		U256([(); 4].map(|()| rng.gen()))
	}

	fn rng_u256_inclusive_bound(rng: &mut impl rand::Rng, bound: std::ops::RangeInclusive<U256>) -> U256 {
		let start = bound.start();
		let end = bound.end();

		let upper_start = (start >> 128).low_u128();
		let upper_end = (end >> 128).low_u128();

		let upper = rand::distributions::Uniform::new_inclusive(upper_start, upper_end).sample(rng);
		let lower = if upper_start < upper && upper < upper_end {
			rng.gen()
		} else {
			rand::distributions::Uniform::new_inclusive(start.low_u128(), end.low_u128()).sample(rng)
		};

		(U256::from(upper) << 128) + U256::from(lower)
	}

	fn rng_u256_numerator_denominator(rng: &mut impl rand::Rng) -> (U256, U256) {
		let numerator = rng_u256(rng);
		(numerator, rng_u256_inclusive_bound(rng, numerator..=U256::MAX))
	}

	for x in std::iter::repeat(()).take(16).into_iter().map(|_| rng_u256(&mut rng)) {
		assert_eq!(
			FloatBetweenZeroAndOne::max(),
			FloatBetweenZeroAndOne::max().mul_div_ceil(x, x)
		);
	}

	for ((x, y), z) in std::iter::repeat(()).take(16).into_iter().map(|_| (
		rng_u256_numerator_denominator(&mut rng),
		rng_u256(&mut rng),
	)) {
		let f = FloatBetweenZeroAndOne::max().mul_div_ceil(x, y);

		assert_eq!((z, z), FloatBetweenZeroAndOne::integer_mul_div(z, &f, &f));
	}

	for ((x, y), z) in (0..16).into_iter().map(|_| (rng_u256_numerator_denominator(&mut rng), rng_u256(&mut rng))) {
		let (floor, ceil) = FloatBetweenZeroAndOne::integer_mul_div(
			z,
			&FloatBetweenZeroAndOne::max().mul_div_ceil(x, y),
			&FloatBetweenZeroAndOne::max(),
		);
		let (bound_floor, bound_ceil) = mul_div(z, x, y);

		assert!(floor >= bound_floor && ceil >= bound_ceil);
	}

	for _ in 0..1024 {
		let initial_value = rng_u256(&mut rng);
		let initial_float = FloatBetweenZeroAndOne::max();

		let (final_value_floor, final_value_ceil, final_float) = (0..rng.gen_range(8, 256)).into_iter().map(|_| rng_u256_numerator_denominator(&mut rng)).fold((initial_value, initial_value, initial_float.clone()), |(value_floor, value_ceil, float), (n, d)| {
			
			(
				mul_div_floor(value_floor, n, d),
				mul_div_ceil(value_ceil, n, d),
				float.mul_div_ceil(n, d),
			)
		});

		println!("{}", final_value_floor);
	
		let final_value_via_float = FloatBetweenZeroAndOne::integer_mul_div(initial_value, &final_float, &initial_float).0;

		assert!(final_value_ceil >= final_value_via_float);
		assert!(final_value_floor <= final_value_via_float);
	}
}

#[test]
fn fee_pips() {
	for bad in [u32::MAX, ONE_IN_PIPS, (ONE_IN_PIPS / 2) + 1] {
		assert!(matches!(PoolState::new(bad), Err(NewError::InvalidFeeAmount)));
	}

	for good in [0, 1, ONE_IN_PIPS / 2] {
		assert_ok!(PoolState::new(good));
	}
}

#[test]
fn mint() {
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>() {
		for good in [MIN_SQRT_PRICE, MAX_SQRT_PRICE - 1, sqrt_price_at_tick(MIN_TICK), sqrt_price_at_tick(MAX_TICK) - 1] {
			let mut pool_state = PoolState::new(0).unwrap();
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), good, 1000.into())), CollectedAmounts::default());
		}

		for bad in [MIN_SQRT_PRICE - 1, MAX_SQRT_PRICE, sqrt_price_at_tick(MIN_TICK) - 1, sqrt_price_at_tick(MAX_TICK)] {
			let mut pool_state = PoolState::new(0).unwrap();
			assert!(matches!(pool_state.collect_and_mint::<SD>(Default::default(), bad, 1000.into()), Err(PositionError::InvalidPrice)));
		}

		for good in [MAX_FIXED_POOL_LIQUIDITY, MAX_FIXED_POOL_LIQUIDITY - 1, 1.into()] {
			let mut pool_state = PoolState::new(0).unwrap();
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price_at_tick(0), good)), CollectedAmounts::default());
		}

		for bad in [MAX_FIXED_POOL_LIQUIDITY + 1, MAX_FIXED_POOL_LIQUIDITY + 2] {
			let mut pool_state = PoolState::new(0).unwrap();
			assert!(matches!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price_at_tick(0), bad), Err(PositionError::Other(MintError::MaximumLiquidity))));
		}
	}

	inner::<ZeroToOne>();
	inner::<OneToZero>();
}

#[test]
fn burn() {
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>() {
		{
			let mut pool_state = PoolState::new(0).unwrap();
			assert!(matches!(pool_state.collect_and_burn::<SD>(Default::default(), MIN_SQRT_PRICE - 1, 1000.into()), Err(PositionError::InvalidPrice)));
			assert!(matches!(pool_state.collect_and_burn::<SD>(Default::default(), MAX_SQRT_PRICE, 1000.into()), Err(PositionError::InvalidPrice)));
		}
		{
			let mut pool_state = PoolState::new(0).unwrap();
			assert!(matches!(pool_state.collect_and_burn::<SD>(Default::default(), sqrt_price_at_tick(120), 1000.into()), Err(PositionError::NonExistent)));
		}
		{
			let mut pool_state = PoolState::new(0).unwrap();
			let sqrt_price = sqrt_price_at_tick(120);
			let amount = U256::from(1000);
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price, amount)), CollectedAmounts::default());
			assert_eq!(assert_ok!(pool_state.collect_and_burn::<SD>(Default::default(), sqrt_price, amount)), (amount, CollectedAmounts::default()));
		}
		{
			let mut pool_state = PoolState::new(0).unwrap();
			let sqrt_price = sqrt_price_at_tick(120);
			let amount = U256::from(1000);
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>([1u8; 32].into(), sqrt_price, 56.into())), CollectedAmounts::default());
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price, amount)), CollectedAmounts::default());
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>([2u8; 32].into(), sqrt_price, 16.into())), CollectedAmounts::default());
			assert_eq!(assert_ok!(pool_state.collect_and_burn::<SD>(Default::default(), sqrt_price, amount)), (amount, CollectedAmounts::default()));
		}
		{
			let mut pool_state = PoolState::new(0).unwrap();
			let sqrt_price = sqrt_price_at_tick(0);
			let amount = U256::from(1000);
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price, amount)), CollectedAmounts::default());
			assert_eq!(pool_state.swap::<SD>(amount, None), (amount, 0.into()));
			assert_eq!(assert_ok!(pool_state.collect_and_burn::<SD>(Default::default(), sqrt_price, 0.into())), (0.into(), CollectedAmounts {
				fees: 0.into(),
				swapped_liquidity: amount,
			}));
		}
		{
			let mut pool_state = PoolState::new(0).unwrap();
			let sqrt_price = sqrt_price_at_tick(0);
			let amount = U256::from(1000);
			let swap = U256::from(500);
			let expected_output = U256::from(500);
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price, amount)), CollectedAmounts::default());
			assert_eq!(pool_state.swap::<SD>(swap, None), (expected_output, 0.into()));
			assert_eq!(assert_ok!(pool_state.collect_and_burn::<SD>(Default::default(), sqrt_price, amount - swap)), (amount - swap, CollectedAmounts {
				fees: 0.into(),
				swapped_liquidity: expected_output,
			}));
		}
	}

	inner::<ZeroToOne>();
	inner::<OneToZero>();
}

#[test]
fn swap() {
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>() {
		{
			let mut pool_state = PoolState::new(0).unwrap();
			let swap = U256::from(20);
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price_at_tick(0), 1000.into())), CollectedAmounts::default());
			assert_eq!(pool_state.swap::<SD>(swap, None), (swap - 1, 0.into()));
		}
		{
			let swap = U256::from(20);
			let output = swap - 1;
			{
				let mut pool_state = PoolState::new(0).unwrap();
				assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price_at_tick(0), 1000.into())), CollectedAmounts::default());
				assert_eq!(pool_state.swap::<SD>(swap, None), (output, 0.into()));
			}
			{
				let mut pool_state = PoolState::new(0).unwrap();
				assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price_at_tick(0), 500.into())), CollectedAmounts::default());
				assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price_at_tick(0), 500.into())), CollectedAmounts::default());
				assert_eq!(pool_state.swap::<SD>(swap, None), (output, 0.into()));
			}
			{
				let mut pool_state = PoolState::new(0).unwrap();		
				assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>([1u8; 32].into(), sqrt_price_at_tick(0), 500.into())), CollectedAmounts::default());
				assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>([2u8; 32].into(), sqrt_price_at_tick(0), 500.into())), CollectedAmounts::default());
				assert_eq!(pool_state.swap::<SD>(swap, None), (output, 0.into()));
			}
		}
		{
			let mut pool_state = PoolState::new(100000).unwrap();
			assert_eq!(assert_ok!(pool_state.collect_and_mint::<SD>(Default::default(), sqrt_price_at_tick(0), 1000.into())), CollectedAmounts::default());
			assert_eq!(pool_state.swap::<SD>(1000.into(), None), (900.into(), 0.into()));
		}
	}

	inner::<ZeroToOne>();
	inner::<OneToZero>();

	// All liquidity, multiple prices
	{
		let mut pool_state = PoolState::new(0).unwrap();
		assert_eq!(assert_ok!(pool_state.collect_and_mint::<ZeroToOne>(Default::default(), sqrt_price_at_tick(0), 100.into())), CollectedAmounts::default());
		assert_eq!(assert_ok!(pool_state.collect_and_mint::<ZeroToOne>(Default::default(), sqrt_price_at_tick(0) * U256::from(4).integer_sqrt(), 100.into())), CollectedAmounts::default());
		assert_eq!(pool_state.swap::<ZeroToOne>(75.into(), None), (150.into(), 0.into()));
	}
	{
		let mut pool_state = PoolState::new(0).unwrap();
		assert_eq!(assert_ok!(pool_state.collect_and_mint::<OneToZero>(Default::default(), sqrt_price_at_tick(0), 100.into())), CollectedAmounts::default());
		assert_eq!(assert_ok!(pool_state.collect_and_mint::<OneToZero>(Default::default(), sqrt_price_at_tick(0) * U256::from(4).integer_sqrt(), 100.into())), CollectedAmounts::default());
		assert_eq!(pool_state.swap::<OneToZero>(180.into(), None), (120.into(), 0.into()));
	}

	// Partial liquidity, multiple prices
	{
		let mut pool_state = PoolState::new(0).unwrap();
		assert_eq!(assert_ok!(pool_state.collect_and_mint::<ZeroToOne>(Default::default(), sqrt_price_at_tick(0), 100.into())), CollectedAmounts::default());
		assert_eq!(assert_ok!(pool_state.collect_and_mint::<ZeroToOne>(Default::default(), sqrt_price_at_tick(0) * U256::from(4).integer_sqrt(), 100.into())), CollectedAmounts::default());
		assert_eq!(pool_state.swap::<ZeroToOne>(150.into(), None), (200.into(), 25.into()));
	}
	{
		let mut pool_state = PoolState::new(0).unwrap();
		assert_eq!(assert_ok!(pool_state.collect_and_mint::<OneToZero>(Default::default(), sqrt_price_at_tick(0), 100.into())), CollectedAmounts::default());
		assert_eq!(assert_ok!(pool_state.collect_and_mint::<OneToZero>(Default::default(), sqrt_price_at_tick(0) * U256::from(4).integer_sqrt(), 100.into())), CollectedAmounts::default());
		assert_eq!(pool_state.swap::<OneToZero>(550.into(), None), (200.into(), 50.into()));
	}
}