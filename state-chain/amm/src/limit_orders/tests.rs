use super::*;

use rand::{SeedableRng, prelude::Distribution};

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

	for _ in 0..64 {
		let initial_value = rng_u256(&mut rng);
		let initial_float = FloatBetweenZeroAndOne::max();

		let (final_value_floor, final_value_ceil, final_float) = (0..64).into_iter().map(|_| rng_u256_numerator_denominator(&mut rng)).fold((initial_value, initial_value, initial_float.clone()), |(value_floor, value_ceil, float), (n, d)| {
			
			(
				mul_div_floor(value_floor, n, d),
				mul_div_ceil(value_ceil, n, d),
				float.mul_div_ceil(n, d),
			)
		});
	
		let final_value_via_float = FloatBetweenZeroAndOne::integer_mul_div(initial_value, &final_float, &initial_float).0;

		assert!(final_value_ceil >= final_value_via_float);
		assert!(final_value_floor <= final_value_via_float);
	}
}