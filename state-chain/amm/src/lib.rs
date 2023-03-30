use std::default;

use cf_utilities::assert_ok;

mod common;
mod limit_orders;
mod range_orders;


use crate::{common::{
	SwapDirection, mul_div_ceil, mul_div_floor, Amount, LiquidityProvider, OneToZero, Side, ZeroToOne, ONE_IN_PIPS, SqrtPriceQ64F96, Tick, sqrt_price_at_tick, MAX_TICK, MIN_TICK, tick_at_sqrt_price, is_sqrt_price_valid,
}, limit_orders::CollectedAmounts};

// use MAX_FIXED_POOL_LIQUIDITY from range_orders.rs
use limit_orders::*;


use primitive_types::U256;


pub struct PoolState {
	pub limit_orders: limit_orders::PoolState,
	pub range_orders: range_orders::PoolState,
}
impl PoolState {
	pub fn swap<SD: common::SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(&mut self, mut amount: Amount) -> (Amount, Amount) {
		let mut total_output_amount = Amount::zero();

		while amount != Amount::zero() {
			let (output_amount, remaining_amount) = match (self.limit_orders.current_sqrt_price::<SD>(), self.range_orders.current_sqrt_price::<SD>()) {
				(Some(limit_order_sqrt_price), Some(range_order_sqrt_price)) => {
					if SD::sqrt_price_op_more_than(limit_order_sqrt_price, range_order_sqrt_price) {
						self.range_orders.swap::<SD>(amount, Some(limit_order_sqrt_price))
					} else {
						self.limit_orders.swap::<SD>(amount, Some(range_order_sqrt_price))
					}
				},
				(Some(_), None) => self.limit_orders.swap::<SD>(amount, None),
				(None, Some(_)) => self.range_orders.swap::<SD>(amount, None),
				(None, None) => break,
			};

			amount = remaining_amount;
			total_output_amount = total_output_amount.saturating_add(output_amount);
		}

		(total_output_amount, amount)
	}
}


// TODO: Add tests for Err(SwapError::InsufficientLiquidity)
// TODO: Add tests to check the returned total_fee_paid value
// TODO: Add tests minting on top of a range order and checking calculated fees.
// TODO: Add tests minting on top of a limit order and checking calculated fees and
// one_minus_swap_perc.
// TODO: Add tests checking exact one_minus_swap_perc values.
// TODO: Add fuzzing tests especially with limit orders (mint, swap, check)

//////////////////////////////////////////////////////////////
// Limit order tests => Adapted uniswap tests for limit orders
//////////////////////////////////////////////////////////////

pub const MIN_TICK_LO_UNISWAP_MEDIUM: Tick = -23016;
pub const MAX_TICK_LO_UNISWAP_MEDIUM: Tick = -MIN_TICK_LO_UNISWAP_MEDIUM;

// NOTE: For now we test the hybrid so we are actually also testing the dual logic. This is so I don't have
//       to modify the tests, are in python I was testing both at the same time
// TODO: Should this be different??
// pub const MIN_TICK_LO: Tick = -665455;
// pub const MAX_TICK_LO: Tick = -MIN_TICK_LO;
pub const MIN_TICK_LO: Tick = MIN_TICK;
pub const MAX_TICK_LO: Tick = MAX_TICK;

// fn mint_pool_lo() -> (PoolState, PoolAssetMap<AmountU256>, LiquidityProvider, Tick, Tick) {
fn mint_pool_lo() -> (PoolState, enum_map::EnumMap<Side, Amount>, Tick, Tick) {
	// encode_price_1_10
	let mut ro_pool = range_orders::PoolState::new(3000, U256::from_dec_str("25054144837504793118650146401").unwrap()).unwrap();
	let mut lo_pool = limit_orders::PoolState::new(3000).unwrap();
	let mut pool = PoolState {
		limit_orders: lo_pool,
		range_orders: ro_pool,
	};
	
	let id: LiquidityProvider = LiquidityProvider::from([0xcf; 32]);
	const MINTED_LIQUIDITY: u128 = 3_161;

	// Setting fallback positions
	let mut collected_amounts_zero =  pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), MIN_TICK_LO, U256::from(3_161)).unwrap();
	let mut collected_amounts_one =  pool.limit_orders.collect_and_mint::<ZeroToOne>(Default::default(), MAX_TICK_LO, U256::from(3_161)).unwrap();

	assert_eq!(collected_amounts_zero, CollectedAmounts::default());
	assert_eq!(collected_amounts_one, CollectedAmounts::default());

	let mut minted_capital: enum_map::EnumMap<Side, Amount> = default::Default::default();
	minted_capital[Side::Zero] = MINTED_LIQUIDITY.into();
	minted_capital[Side::One] = MINTED_LIQUIDITY.into();


	assert_eq!(pool.range_orders.current_tick, -23028);

	// Closest ticks to the initialized pool tick with TICK_SPACING_MEDIUM_UNISWAP
	let close_initick_rdown: Tick = -23040;
	let close_initick_rup: Tick = -22980;

	(pool, minted_capital, close_initick_rdown, close_initick_rup)
	// (pool, minted_capital_accum, id, close_initick_rdown, close_initick_rup)
}

#[test]
fn test_trialmint_lo() {
	let (mut pool, _, _, _) = mint_pool_lo();
	assert_ok!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), MIN_TICK_LO, U256::from(3_161)));
	assert_ok!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), MAX_TICK_LO, U256::from(3_161)));
	assert_ok!(pool.limit_orders.collect_and_mint::<ZeroToOne>(Default::default(), MIN_TICK_LO, U256::from(3_161)));
	assert_ok!(pool.limit_orders.collect_and_mint::<ZeroToOne>(Default::default(), MAX_TICK_LO, U256::from(3_161)));
}

// Minting

#[test]
fn test_mint_err_lo() {

	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), -887273, U256::from(3_161)),Err(PositionError::InvalidPrice));
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(),  MIN_TICK_LO - 1, U256::from(3_161)),Err(PositionError::InvalidPrice));
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(),  MAX_TICK_LO + 1, U256::from(3_161)),Err(PositionError::InvalidPrice));
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), MIN_TICK_LO+1, MAX_FIXED_POOL_LIQUIDITY+1),Err(PositionError::Other(MintError::MaximumLiquidity)));
		assert_ok!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), MAX_TICK_LO-1, MAX_FIXED_POOL_LIQUIDITY));
	}

	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);

}

#[test]
fn test_mint_err_tickmax_lo() {
	
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {

		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), MAX_TICK_LO-1, U256::from(1000)).unwrap(), CollectedAmounts::default());
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), MIN_TICK+1, U256::from(1000)).unwrap(), CollectedAmounts::default());

		assert_eq!((pool.limit_orders.collect_and_mint::<SD>(Default::default(), MAX_TICK_LO-1, MAX_FIXED_POOL_LIQUIDITY - 1000 + 1)),Err(PositionError::Other(MintError::MaximumLiquidity)));
		assert_eq!((pool.limit_orders.collect_and_mint::<SD>(Default::default(), MIN_TICK_LO+1, MAX_FIXED_POOL_LIQUIDITY - 1000 + 1)),Err(PositionError::Other(MintError::MaximumLiquidity)));

		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), MAX_TICK_LO-1, MAX_FIXED_POOL_LIQUIDITY - 1000).unwrap(), CollectedAmounts::default());
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), MIN_TICK_LO+1, MAX_FIXED_POOL_LIQUIDITY - 1000).unwrap(), CollectedAmounts::default());

		}
		let (mut pool, _, _, _) = mint_pool_lo();
		inner::<ZeroToOne>(&mut pool);
		inner::<OneToZero>(&mut pool);
	}

// Success cases

#[test]
fn test_balances_lo() {
	let (_, minted_capital, _, _) = mint_pool_lo();
	// Check "balances"
	assert_eq!(minted_capital[Side::Zero], U256::from(3_161));
	assert_eq!(minted_capital[!Side::Zero], U256::from(3_161));
}



#[test]
fn test_mint_one_side_lo() {
	
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {
		let liquidity: Side = !SD::INPUT_SIDE;

		// Adding one unit to this liquidity to make them different for testing purposes
		assert_ok!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), pool.range_orders.current_tick, U256::from(3161)));

		// TO CHECK: Here we don't even get a minted_capital value so we check that the tick and position have that amount
		assert_eq!(pool.limit_orders.fixed_pools[!SD::INPUT_SIDE].get(&sqrt_price_at_tick(pool.range_orders.current_tick)).unwrap().available, U256::from(3161));

		assert_ok!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), pool.range_orders.current_tick, U256::from(3161)));
		assert_eq!(pool.limit_orders.fixed_pools[!SD::INPUT_SIDE].get(&sqrt_price_at_tick(pool.range_orders.current_tick)).unwrap().available, U256::from(3161*2));
	}

	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);

}

#[test]
fn test_initial_tick_lo() {
	let (pool, _, _, _) = mint_pool_lo();
	// Check current tick - -imit orders have not altered the tick
	assert_eq!(pool.range_orders.current_tick, -23_028);
}

#[test]
// Above current price
fn test_transfer_onetoken_only_lo() {

	const MINTED_LIQUIDITY: u128 = 10_000;

	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {
		let (mut pool, _, _, _) = mint_pool_lo();

		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), -22980, MINTED_LIQUIDITY.into()).unwrap(), CollectedAmounts::default());

		assert_eq!(pool.limit_orders.fixed_pools[!SD::INPUT_SIDE].get(&sqrt_price_at_tick(-22980)).unwrap().available, MINTED_LIQUIDITY.into());
		assert!(!pool.limit_orders.fixed_pools[SD::INPUT_SIDE].contains_key(&sqrt_price_at_tick(-22980)));
	}

	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
}

#[test]
fn test_maxtick_maxleverage_lo() {
	let (mut pool, mut minted_capital_accum, _, _) = mint_pool_lo();

	assert_eq!(pool.limit_orders.collect_and_mint::<ZeroToOne>(Default::default(), MAX_TICK_LO, U256::from_dec_str("5070602400912917605986812821504").unwrap()).unwrap(), CollectedAmounts::default());
	assert_eq!(get_fixed_pool(&pool, Side::One, MAX_TICK_LO).unwrap().available, U256::from_dec_str("5070602400912917605986812821504").unwrap() + 3_161);

	assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), MAX_TICK_LO, U256::from_dec_str("5070602400912917605986812821504").unwrap()).unwrap(), CollectedAmounts::default());
	assert_eq!(get_fixed_pool(&pool, Side::Zero, MAX_TICK_LO).unwrap().available, U256::from_dec_str("5070602400912917605986812821504").unwrap());

}

#[test]
fn test_maxtick_lo() {
	let (mut pool, mut minted_capital_accum, _, _) = mint_pool_lo();

	assert_eq!(pool.limit_orders.collect_and_mint::<ZeroToOne>(Default::default(), MAX_TICK_LO, U256::from(10000)).unwrap(), CollectedAmounts::default());
	assert_eq!(get_fixed_pool(&pool, Side::One, MAX_TICK_LO).unwrap().available, U256::from(10000+ 3_161));


	assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), MAX_TICK_LO, U256::from(10000)).unwrap(), CollectedAmounts::default());
	assert_eq!(get_fixed_pool(&pool, Side::Zero, MAX_TICK_LO).unwrap().available, U256::from(10000));
}

#[test]
fn test_removing_works_lo() {
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {

		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), -240, U256::from(10000)).unwrap(), CollectedAmounts::default());
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), 0, U256::from(10001)).unwrap(), CollectedAmounts::default());



		let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<SD>(Default::default(), -240, U256::from(10000)).unwrap();
		assert_eq!(amount, U256::from(10000));
		assert_eq!(collected_amounts, CollectedAmounts::default());

		let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<SD>(Default::default(), 0, U256::from(10001)).unwrap();
		assert_eq!(amount, U256::from(10001));
		assert_eq!(collected_amounts, CollectedAmounts::default());
	}
	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
}

#[test]
fn test_removing_works_twosteps_lo() {

	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {

		for i in [-240, 0] {
			assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), i, U256::from(10000)).unwrap(), CollectedAmounts::default());

			let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<SD>(Default::default(), i, U256::from(10000/2)).unwrap();
			assert_eq!(amount, U256::from(10000/2));
			assert_eq!(collected_amounts, CollectedAmounts::default());

		}
	}
	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
}

fn get_fixed_pool (pool: &PoolState, side: Side, tick: Tick) -> Option<&FixedPool> {
	pool.limit_orders.fixed_pools[side].get(&sqrt_price_at_tick(tick))
}

// fn get_tickinfo_limit_orders<'a>(
// 	pool: &'a PoolState,
// 	asset: PoolSide,
// 	tick: &'a Tick,
// ) -> Option<&'a TickInfoLimitOrder> {
// 	if asset == Side::Zero {
// 		return pool.liquidity_map_base_lo.get(tick)
// 	}
// 	pool.liquidity_map_pair_lo.get(tick)
// }

// fn get_liquiditymap_lo(pool: &PoolState, asset: PoolSide) -> &BTreeMap<Tick, TickInfoLimitOrder> {
// 	if asset == Side::Zero {
// 		return &pool.liquidity_map_base_lo
// 	}
// 	&pool.liquidity_map_pair_lo
// }

#[test]
fn test_addliquidityto_liquiditygross_lo() {
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {
		let mut pool_instance_number:u128 = pool.limit_orders.next_pool_instance;

		for i in [-240, 0] {
			assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), i, U256::from(100)).unwrap(), CollectedAmounts::default());

			let fixed_pool = get_fixed_pool(&pool, !SD::INPUT_SIDE, i).unwrap();

			assert_eq!(fixed_pool.available, U256::from(100));
			assert_eq!(fixed_pool.pool_instance, pool_instance_number);
			assert_eq!(fixed_pool.percent_remaining, FloatBetweenZeroAndOne::max());
			let pool_instance_clone = pool_instance_number.clone();
			pool_instance_number += 1;
			assert_eq!(pool.limit_orders.next_pool_instance, pool_instance_number);

			// No liquidityGross === tick doesn't exist
			assert!(get_fixed_pool(&pool, !SD::INPUT_SIDE, 1).is_none());
			assert!(get_fixed_pool(&pool, !SD::INPUT_SIDE, 2).is_none());


			assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), i, U256::from(150)).unwrap(), CollectedAmounts::default());

			// pool_instance_number shouldn't have changed
			let fixed_pool = get_fixed_pool(&pool, !SD::INPUT_SIDE, i).unwrap();
			assert_eq!(fixed_pool.available, U256::from(100+150));
			assert_eq!(fixed_pool.pool_instance, pool_instance_clone);
			assert_eq!(fixed_pool.percent_remaining, FloatBetweenZeroAndOne::max());
			assert_eq!(pool.limit_orders.next_pool_instance, pool_instance_number);

			assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), i+1, U256::from(150)).unwrap(), CollectedAmounts::default());
			let fixed_pool = get_fixed_pool(&pool, !SD::INPUT_SIDE, i+1).unwrap();
			assert_eq!(fixed_pool.available, U256::from(150));
			assert_eq!(fixed_pool.pool_instance, pool_instance_number);
			assert_eq!(fixed_pool.percent_remaining, FloatBetweenZeroAndOne::max());
			pool_instance_number += 1;
			assert_eq!(pool.limit_orders.next_pool_instance, pool_instance_number);
		}
	}
	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
}

#[test]
fn test_remove_liquidity_liquiditygross_lo() {

	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {
		let mut pool_instance_number:u128 = pool.limit_orders.next_pool_instance;

		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), -240, U256::from(100)).unwrap(), CollectedAmounts::default());
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), 0, U256::from(40)).unwrap(), CollectedAmounts::default());
		pool_instance_number += 2;

		assert_eq!(get_fixed_pool(&pool, !SD::INPUT_SIDE, -240).unwrap().available, U256::from(100));
		assert_eq!(get_fixed_pool(&pool, !SD::INPUT_SIDE, 0).unwrap().available, U256::from(40));

		let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<SD>(Default::default(), -240, U256::from(90)).unwrap();

		let fixed_pool = get_fixed_pool(&pool, !SD::INPUT_SIDE, -240).unwrap();
		assert_eq!(fixed_pool.available, U256::from(10));
		assert_eq!(fixed_pool.pool_instance, pool_instance_number - 2);
		assert_eq!(fixed_pool.percent_remaining, FloatBetweenZeroAndOne::max());
		assert_eq!(pool.limit_orders.next_pool_instance, pool_instance_number);

		let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<SD>(Default::default(), 0, U256::from(25)).unwrap();

		let fixed_pool = get_fixed_pool(&pool, !SD::INPUT_SIDE, 0).unwrap();
		assert_eq!(fixed_pool.available, U256::from(15));
		assert_eq!(fixed_pool.pool_instance, pool_instance_number - 1);
		assert_eq!(fixed_pool.percent_remaining, FloatBetweenZeroAndOne::max());
		assert_eq!(pool.limit_orders.next_pool_instance, pool_instance_number);
	}
	
	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
}

#[test]
fn test_clearstick_ifpositionremoved_lo() {

	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {

		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), -240, U256::from(100)).unwrap(), CollectedAmounts::default());
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), 0, U256::from(100)).unwrap(), CollectedAmounts::default());

		pool.limit_orders.collect_and_burn::<SD>(Default::default(), -240, U256::from(100)).unwrap();

		assert!(get_fixed_pool(&pool, !SD::INPUT_SIDE, -240).is_none());
		assert_ok!(get_fixed_pool(&pool, !SD::INPUT_SIDE, 0));

		pool.limit_orders.collect_and_burn::<SD>(Default::default(), 0, U256::from(100)).unwrap();

		assert!(get_fixed_pool(&pool, !SD::INPUT_SIDE, -240).is_none());
		assert!(get_fixed_pool(&pool, !SD::INPUT_SIDE, 0).is_none());


	}
	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
}

#[test]
fn test_clears_onlyunused_lo() {

	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {
		let id2: LiquidityProvider = LiquidityProvider::from([0xce; 32]);

		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), -240, U256::from(100)).unwrap(), CollectedAmounts::default());
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), 0, U256::from(100)).unwrap(), CollectedAmounts::default());

		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(id2, -1, U256::from(250)).unwrap(), CollectedAmounts::default());
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(id2, 0, U256::from(250)).unwrap(), CollectedAmounts::default());

		assert_ok!(get_fixed_pool(&pool, !SD::INPUT_SIDE, -240));
		assert_ok!(get_fixed_pool(&pool, !SD::INPUT_SIDE, -1));
		assert_ok!(get_fixed_pool(&pool, !SD::INPUT_SIDE, 0));

		assert!(pool.limit_orders.collect_and_burn::<SD>(id2, -240, U256::from(100)).is_err());
		pool.limit_orders.collect_and_burn::<SD>(Default::default(), -240, U256::from(100)).unwrap();
		pool.limit_orders.collect_and_burn::<SD>(Default::default(), 0, U256::from(100)).unwrap();

		assert!(get_fixed_pool(&pool, !SD::INPUT_SIDE, -240).is_none());

		let fixed_pool = get_fixed_pool(&pool, !SD::INPUT_SIDE, -1).unwrap();
		assert_eq!(fixed_pool.available, U256::from(250));
		assert_eq!(fixed_pool.percent_remaining, FloatBetweenZeroAndOne::max());

	}
	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
}

// // Including current price

// #[test]
// fn price_within_range_lo() {
// 	let (mut pool, minted_capital_accum, id,_,_) = mint_pool_lo();
//
// 	pool.mint_limit_order_base(
// 		id,
// 		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
// 		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
// 		100,
// 		|_| {
//
// 			Ok::<(), ()>(())
// 		},
// 	)
// 	.unwrap();
//

// 	assert_eq!(minted_capital[Side::Zero], U256::from(317));
// 	assert_eq!(minted_capital[!Side::Zero], U256::from(32));

// 	assert_eq!(
// 		minted_capital_accum[Side::Zero] + minted_capital[Side::Zero],
// 		U256::from(9_996 + 317)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!Side::Zero] + minted_capital[!Side::Zero],
// 		U256::from(1_000 + 32)
// 	);
// }

// #[test]
// fn initializes_lowertick_lo() {
// 	let (mut pool, _, id,_,_) = mint_pool_lo();
// 	pool.mint_limit_order_base(
// 		id,
// 		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
// 		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
// 		100,
// 		|_| {Ok::<(), ()>(())},
// 	)
// 	.unwrap();
// 	assert_eq!(
// 		pool.liquidity_map
// 			.get(&(MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM))
// 			.unwrap()
// 			.liquidity_gross,
// 		100
// 	);
// }

// #[test]
// fn initializes_uppertick_lo() {
// 	let (mut pool, _, id,_,_) = mint_pool_lo();
// 	pool.mint_limit_order_base(
// 		id,
// 		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
// 		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
// 		100,
// 		|_| {Ok::<(), ()>(())},
// 	)
// 	.unwrap();
// 	assert_eq!(
// 		pool.liquidity_map
// 			.get(&(MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM))
// 			.unwrap()
// 			.liquidity_gross,
// 		100
// 	);
// }

// #[test]
// fn minmax_tick_lo() {
// 	let (mut pool, minted_capital_accum, id,_,_) = mint_pool_lo();
//
// 	pool.mint_limit_order_base(
// 		id,
// 		MIN_TICK_UNISWAP_MEDIUM,
// 		MAX_TICK_UNISWAP_MEDIUM,
// 		10000,
// 		|_| {
//
// 			Ok::<(), ()>(())
// 		},
// 	)
// 	.unwrap();
//

// 	assert_eq!(minted_capital[Side::Zero], U256::from(31623));
// 	assert_eq!(minted_capital[!Side::Zero], U256::from(3163));

// 	assert_eq!(
// 		minted_capital_accum[Side::Zero] + minted_capital[Side::Zero],
// 		U256::from(9_996 + 31623)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!Side::Zero] + minted_capital[!Side::Zero],
// 		U256::from(1_000 + 3163)
// 	);
// }


// // Below current price

// #[test]
// fn transfer_token1_only_lo() {
// 	let (mut pool, minted_capital_accum, id,_,_) = mint_pool_lo();
//
// 	pool.mint_limit_order_base(id.clone(), -46080, -23040, 10000, |_| {
//
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();
//

// 	assert_eq!(minted_capital[Side::Zero], U256::from(0));
// 	assert_eq!(minted_capital[!Side::Zero], U256::from(2162));

// 	assert_eq!(
// 		minted_capital_accum[Side::Zero] + minted_capital[Side::Zero],
// 		U256::from(9_996)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!Side::Zero] + minted_capital[!Side::Zero],
// 		U256::from(1_000 + 2162)
// 	);
// }

// #[test]
// fn mintick_maxleverage_lo() {
// 	let (mut pool, minted_capital_accum, id,_,_) = mint_pool_lo();
//
// 	pool.mint_limit_order_base(
// 		id,
// 		MIN_TICK_UNISWAP_MEDIUM,
// 		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
// 		5070602400912917605986812821504, /* 2**102 */
// 		|_| {
//
// 			Ok::<(), ()>(())
// 		},
// 	)
// 	.unwrap();
//

// 	assert_eq!(minted_capital[Side::Zero], U256::from(0));
// 	assert_eq!(minted_capital[!Side::Zero], U256::from(828011520));

// 	assert_eq!(
// 		minted_capital_accum[Side::Zero] + minted_capital[Side::Zero],
// 		U256::from(9_996)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!Side::Zero] + minted_capital[!Side::Zero],
// 		U256::from(1_000 + 828011520)
// 	);
// }

// #[test]
// fn mintick_lo() {
// 	let (mut pool, minted_capital_accum, id,_,_) = mint_pool_lo();
//
// 	pool.mint_limit_order_base(id.clone(), MIN_TICK_UNISWAP_MEDIUM, -23040, 10000,
// |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();
//

// 	assert_eq!(minted_capital[Side::Zero], U256::from(0));
// 	assert_eq!(minted_capital[!Side::Zero], U256::from(3161));

// 	assert_eq!(
// 		minted_capital_accum[Side::Zero] + minted_capital[Side::Zero],
// 		U256::from(9_996)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!Side::Zero] + minted_capital[!Side::Zero],
// 		U256::from(1_000 + 3161)
// 	);
// }

#[test]
fn removing_works_1_lo() {

	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {

		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), -46080, U256::from(10000)).unwrap(), CollectedAmounts::default());
		assert_eq!(pool.limit_orders.collect_and_mint::<SD>(Default::default(), -46020, U256::from(10001)).unwrap(), CollectedAmounts::default());

		let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<SD>(Default::default(), -46080, U256::from(10000)).unwrap();
		assert_eq!(amount, U256::from(10000));
		assert_eq!(collected_amounts, CollectedAmounts::default());

		assert!(get_fixed_pool(&pool, !SD::INPUT_SIDE, -46080).is_none());
		assert_ok!(get_fixed_pool(&pool, !SD::INPUT_SIDE, -46020));

		let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<SD>(Default::default(), -46020, U256::from(10001)).unwrap();
		assert_eq!(amount, U256::from(10001));
		assert_eq!(collected_amounts, CollectedAmounts::default());

		assert!(get_fixed_pool(&pool, !SD::INPUT_SIDE, -46020).is_none());

	}
	let (mut pool, _, _, _) = mint_pool_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
}


pub fn get_position(pool: &PoolState, side: Side, tick: Tick, lp: LiquidityProvider) -> Option<&limit_orders::Position> {
	pool.limit_orders.positions[side].get(&(sqrt_price_at_tick(tick), lp))
}

pub fn expandto18decimals(amount: u128) -> U256 {
	U256::from(amount) * U256::from(10).pow(U256::from_dec_str("18").unwrap())
}

#[test]
fn poke_uninitialized_position_lo() {

	let (mut pool, _, initick_rdown, initick_rup) = mint_pool_lo();

	let id2: LiquidityProvider = LiquidityProvider::from([0xce; 32]);

	assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(id2, initick_rdown, expandto18decimals(1)).unwrap(), CollectedAmounts::default());
	assert_eq!(pool.limit_orders.collect_and_mint::<ZeroToOne>(id2, initick_rup, expandto18decimals(1)).unwrap(), CollectedAmounts::default());

	let swap_input_amount_zero: U256 = expandto18decimals(1)/10;
	let swap_input_amount_one: U256 = expandto18decimals(1)/100;

	// Pool initialized at 1:10 => tick -22980
	let (amount_out_one, amount) = pool.swap::<ZeroToOne>(swap_input_amount_zero);
	assert_eq!(amount, U256::from(0));
	assert!(amount_out_one < swap_input_amount_zero);
	let (amount_out_zero, amount) = pool.swap::<OneToZero>(swap_input_amount_one);
	assert_eq!(amount, U256::from(0));
	assert!(amount_out_zero > swap_input_amount_one);

	// Check another LP can't collect the position
	assert_eq!(pool.limit_orders.collect_and_burn::<OneToZero>(Default::default(), initick_rdown, U256::from(0)), Err(PositionError::NonExistent));
	assert_eq!(pool.limit_orders.collect_and_burn::<ZeroToOne>(Default::default(), initick_rup, U256::from(0)), Err(PositionError::NonExistent));

	// Check that another LP can mint on the same tick but doesn't get any fees back
	assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), initick_rdown, U256::from(1)).unwrap(), CollectedAmounts::default());
	assert_eq!(pool.limit_orders.collect_and_mint::<ZeroToOne>(Default::default(), initick_rup, U256::from(1)).unwrap(), CollectedAmounts::default());
	
	let position = get_position(&pool, Side::Zero, initick_rdown, Default::default()).unwrap();
	assert_eq!(position.amount, U256::from(1));
	// TODO: Add check percent_remaining < FloatBetweenZeroAndOne::max()
	assert!(position.percent_remaining != FloatBetweenZeroAndOne::max());

	// Check that old positions have not been modified
	let position = get_position(&pool, Side::Zero, initick_rdown, id2).unwrap();
	assert_eq!(position.amount,expandto18decimals(1));
	assert_eq!(position.percent_remaining , FloatBetweenZeroAndOne::max());
	let position = get_position(&pool, Side::One, initick_rup, id2).unwrap();
	assert_eq!(position.amount,expandto18decimals(1));
	assert_eq!(position.percent_remaining , FloatBetweenZeroAndOne::max());

	// Check the fees accrued in each of the two swaps. In this case we don't have feeGrowthInside
	let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<ZeroToOne>(id2, initick_rup, U256::from(1)).unwrap();
	// TO CHECK: Fees collected = 0.3% of the swap input with a margin of 1 due to rounding. Also enforcing that collected_amounts <= max_fee calculated.
	// Is 16 units of rounding acceptable?
	assert!(swap_input_amount_zero * 3 / 1000 - collected_amounts.fees <= U256::from(1));
	assert!(swap_input_amount_zero - (collected_amounts.swapped_liquidity + collected_amounts.fees) <= U256::from(16));
	assert_eq!(amount, U256::from(1));
	// TO CHECK: Values expected are obtained with the Python implementation
	assert_eq!(collected_amounts.fees, U256::from_dec_str("299999999999999").unwrap());

	let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<OneToZero>(id2, initick_rdown, U256::from(1)).unwrap();
	assert!(swap_input_amount_one * 3 / 1000 - collected_amounts.fees <= U256::from(1));
	assert!(swap_input_amount_one - (collected_amounts.swapped_liquidity + collected_amounts.fees) <= U256::from(16));
	assert_eq!(amount, U256::from(1));
	assert_eq!(collected_amounts.fees, U256::from_dec_str("29999999999999").unwrap());
}



// #Burn
pub const TICKSPACING_UNISWAP_MEDIUM: Tick = 60;
pub const INITIALIZE_LIQUIDITY_AMOUNT: u128 = 2000000000000000000_u128;

fn mediumpool_initialized_zerotick_lo(
) -> (PoolState, Tick, Tick) {
	// encode_price_1_1
	let mut ro_pool = range_orders::PoolState::new(3000, U256::from_dec_str("79228162514264337593543950336").unwrap()).unwrap();
	let mut lo_pool = limit_orders::PoolState::new(3000).unwrap();
	let mut pool = PoolState {
		limit_orders: lo_pool,
		range_orders: ro_pool,
	};

	let initick_rdown = pool.range_orders.current_tick;
	let initick_rup = pool.range_orders.current_tick + TICKSPACING_UNISWAP_MEDIUM;

	assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), initick_rdown, INITIALIZE_LIQUIDITY_AMOUNT.into()).unwrap(), CollectedAmounts::default());
	assert_eq!(pool.limit_orders.collect_and_mint::<ZeroToOne>(Default::default(), initick_rup, INITIALIZE_LIQUIDITY_AMOUNT.into()).unwrap(), CollectedAmounts::default());

	(pool, initick_rdown, initick_rup)
}

fn estimate_lo_outcome(pool: &PoolState, tick: Tick, amount: U256, side: Side) -> U256 {
	let amount_minus_fees = mul_div_floor(
		amount,
		U256::from(ONE_IN_PIPS - pool.limit_orders.fee_pips),
		U256::from(ONE_IN_PIPS));
	match side {
		Side::Zero => mul_div_floor(amount_minus_fees
		, U256::one() << 128, sqrt_price_to_price(sqrt_price_at_tick(tick))),
		Side::One => mul_div_floor(amount_minus_fees, sqrt_price_to_price(sqrt_price_at_tick(tick)), U256::one() << 128)
	}

}
	

#[test]
fn notclearposition_ifnomoreliquidity_lo() {
	let (mut pool, initick_rdown, initick_rup) = mediumpool_initialized_zerotick_lo();
	let id2 = LiquidityProvider::from([0xce; 32]);

	let ini_tick = pool.range_orders.current_tick;


	let (amount_out_one, amount) = pool.swap::<ZeroToOne>(expandto18decimals(1));
	assert_eq!(amount, U256::from(0));
	// Swapped slightly better than the 1:1 price. Also check against Python implementation
	assert!(estimate_lo_outcome(&pool, initick_rup, expandto18decimals(1), Side::One) - amount_out_one <= U256::from(1));
	assert_eq!(amount_out_one, U256::from_dec_str("1002999681066011709").unwrap());

	let (amount_out_zero, amount) = pool.swap::<OneToZero>(expandto18decimals(1));
	assert_eq!(amount, U256::from(0));
	// Swapped slightly better than the 1:1 price. Also check against Python implementation
	assert!(estimate_lo_outcome(&pool, initick_rdown, expandto18decimals(1), Side::Zero) - amount_out_zero <= U256::from(1));
	assert_eq!(amount_out_zero, U256::from_dec_str("996999999999999999").unwrap());

	// Collect
	let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<ZeroToOne>(Default::default(), initick_rup, U256::from(0)).unwrap();
	assert!(expandto18decimals(1) * 3 / 1000 - collected_amounts.fees <= U256::from(1));
	assert!(expandto18decimals(1) - (collected_amounts.swapped_liquidity + collected_amounts.fees) <= U256::from(3));
	assert_eq!(collected_amounts.fees, U256::from_dec_str("2999999999999999").unwrap());

	let (amount, collected_amounts) = pool.limit_orders.collect_and_burn::<OneToZero>(Default::default(), initick_rdown, U256::from(0)).unwrap();
	assert!(expandto18decimals(1) * 3 / 1000 - collected_amounts.fees <= U256::from(1));
	assert!(expandto18decimals(1) - (collected_amounts.swapped_liquidity + collected_amounts.fees) <= U256::from(3));
	assert_eq!(collected_amounts.fees, U256::from_dec_str("2999999999999999").unwrap());
}


// Miscellaneous mint tests

pub const TICKSPACING_UNISWAP_LOW: Tick = 10;
pub const MIN_TICK_UNISWAP_LOW: Tick = -887220;
pub const MAX_TICK_UNISWAP_LOW: Tick = -MIN_TICK_UNISWAP_LOW;

// // Low Fee, tickSpacing = 10, 1:1 price
fn lowpool_initialized_zerotick_lo() -> (PoolState, Tick, Tick)
{	
	// encode_price_1_1
	let mut ro_pool = range_orders::PoolState::new(500, U256::from_dec_str("79228162514264337593543950336").unwrap()).unwrap();
	let mut lo_pool = limit_orders::PoolState::new(500).unwrap();
	let mut pool = PoolState {
		limit_orders: lo_pool,
		range_orders: ro_pool,
	};

	let initick_rdown = pool.range_orders.current_tick;
	let initick_rup = pool.range_orders.current_tick + TICKSPACING_UNISWAP_LOW;

	assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), initick_rdown, INITIALIZE_LIQUIDITY_AMOUNT.into()).unwrap(), CollectedAmounts::default());
	assert_eq!(pool.limit_orders.collect_and_mint::<ZeroToOne>(Default::default(), initick_rup, INITIALIZE_LIQUIDITY_AMOUNT.into()).unwrap(), CollectedAmounts::default());

	(pool, initick_rdown, initick_rup)
}

#[test]
fn test_mint_rightofcurrentprice_lo() {
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {
		let liquiditybefore = pool.range_orders.current_liquidity;
		for tick in TICKSPACING_UNISWAP_LOW..TICKSPACING_UNISWAP_LOW * 2 {
			assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), tick, INITIALIZE_LIQUIDITY_AMOUNT.into()).unwrap(), CollectedAmounts::default());
			assert_eq!(pool.range_orders.current_liquidity, liquiditybefore);
		}
	}

	let (mut pool, _, _) = lowpool_initialized_zerotick_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
}

#[test]
fn test_mint_leftofcurrentprice_lo() {
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {
		let liquiditybefore = pool.range_orders.current_liquidity;
		for tick in -TICKSPACING_UNISWAP_LOW*2..-TICKSPACING_UNISWAP_LOW {
			assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), tick, INITIALIZE_LIQUIDITY_AMOUNT.into()).unwrap(), CollectedAmounts::default());
			assert_eq!(pool.range_orders.current_liquidity, liquiditybefore);
		}
	}

	let (mut pool, _, _) = lowpool_initialized_zerotick_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
	
}

#[test]
fn test_mint_withincurrentprice_lo() {
	fn inner<SD: SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(pool: &mut PoolState) {
		let liquiditybefore = pool.range_orders.current_liquidity;
		for tick in -TICKSPACING_UNISWAP_LOW..TICKSPACING_UNISWAP_LOW {
			assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), tick, INITIALIZE_LIQUIDITY_AMOUNT.into()).unwrap(), CollectedAmounts::default());
			assert_eq!(pool.range_orders.current_liquidity, liquiditybefore);
		}
	}

	let (mut pool, _, _) = lowpool_initialized_zerotick_lo();
	inner::<ZeroToOne>(&mut pool);
	inner::<OneToZero>(&mut pool);
	
}

#[test]
fn test_cannotremove_morethanposition_lo() {
	let (mut pool,initick_rdown, initick_rup) = lowpool_initialized_zerotick_lo();

	assert_eq!(pool.limit_orders.collect_and_burn::<OneToZero>(Default::default(), initick_rdown, (INITIALIZE_LIQUIDITY_AMOUNT + 1).into()), Err(PositionError::Other(BurnError::PositionLacksLiquidity)));
	assert_eq!(pool.limit_orders.collect_and_burn::<ZeroToOne>(Default::default(), initick_rup, (INITIALIZE_LIQUIDITY_AMOUNT + 1).into()), Err(PositionError::Other(BurnError::PositionLacksLiquidity)));
}

#[test]
fn test_collectfees_withincurrentprice_lo() {
	let (mut pool, _, _) = lowpool_initialized_zerotick_lo();

	let liquidity_delta: U256 = expandto18decimals(1000);
	let lowtick: Tick = -TICKSPACING_UNISWAP_LOW * 100;
	let uptick: Tick = TICKSPACING_UNISWAP_LOW * 100;

	assert_eq!(pool.limit_orders.collect_and_mint::<OneToZero>(Default::default(), lowtick, liquidity_delta.into()).unwrap(), CollectedAmounts::default());
	assert_eq!(pool.limit_orders.collect_and_mint::<ZeroToOne>(Default::default(), uptick, liquidity_delta.into()).unwrap(), CollectedAmounts::default());

	let before_pool_low_lo = get_fixed_pool(&pool, Side::Zero, lowtick).unwrap().clone();
	let before_pool_up_lo = get_fixed_pool(&pool, Side::One, uptick).unwrap().clone();

	let (amount_out_one, amount) = pool.swap::<ZeroToOne>(expandto18decimals(1));
	assert!(estimate_lo_outcome(&pool, uptick, expandto18decimals(1), Side::One) - amount_out_one <= U256::from(1));

	assert_ne!(amount_out_one, U256::zero());
	assert_eq!(amount, U256::zero());

	// Poke pos0
	let (_, collected_amounts) =  pool.limit_orders.collect_and_burn::<OneToZero>(Default::default(), lowtick, 0.into()).unwrap();

	assert_eq!(before_pool_low_lo, *get_fixed_pool(&pool, Side::Zero, lowtick).unwrap());
	assert_eq!(collected_amounts, CollectedAmounts::default());
	assert_ok!(get_position(&pool, Side::Zero, lowtick, Default::default()));

	// Check fixed_pool and poke pos1
	let fixed_pool = get_fixed_pool(&pool, Side::One, uptick).unwrap();
	assert_eq!(fixed_pool.pool_instance, before_pool_up_lo.pool_instance);
	assert!(fixed_pool.available<before_pool_up_lo.available);
	// TODO: Check <
	assert!(fixed_pool.percent_remaining!=before_pool_up_lo.percent_remaining);

	let (_, collected_amounts) =  pool.limit_orders.collect_and_burn::<ZeroToOne>(Default::default(), uptick, 0.into()).unwrap();
	let position = get_position(&pool, Side::One, uptick, Default::default()).unwrap();

	assert!(expandto18decimals(1) * 5 / 10000 - collected_amounts.fees <= U256::from(1));
	assert!(expandto18decimals(1) - (collected_amounts.swapped_liquidity + collected_amounts.fees) <= U256::from(1));
	assert_eq!(collected_amounts.fees, U256::from_dec_str("499999999999999").unwrap());
}

// Post initialize at medium fee

#[test]
fn test_initial_liquidity_lo() {
	let (pool, initick_rdown, initick_rup) = mediumpool_initialized_zerotick_lo();

	assert_eq!(get_fixed_pool(&pool, Side::Zero, initick_rdown).unwrap().available + get_fixed_pool(&pool, Side::One, initick_rup).unwrap().available, (INITIALIZE_LIQUIDITY_AMOUNT * 2).into());
}

// #[test]
// fn test_returns_insupply_inrange_lo() {
// 	let (mut pool, _, _, _) = mediumpool_initialized_zerotick_lo();
// 	pool.mint_limit_order(
// 		id.clone(),
// 		-TICKSPACING_UNISWAP_MEDIUM,
// 		expandto18decimals(3).as_u128(),
// 		Side::Zero,
// 		|_| Ok::<(), ()>(()),
// 	)
// 	.unwrap();
// 	pool.mint_limit_order(
// 		id,
// 		TICKSPACING_UNISWAP_MEDIUM,
// 		expandto18decimals(2).as_u128(),
// 		Side::One,
// 		|_| Ok::<(), ()>(()),
// 	)
// 	.unwrap();
// 	assert_eq!(
// 		get_tickinfo_limit_orders(&pool, Side::Zero, &-TICKSPACING_UNISWAP_MEDIUM)
// 			.unwrap()
// 			.liquidity_gross +
// 			get_tickinfo_limit_orders(&pool, Side::One, &TICKSPACING_UNISWAP_MEDIUM)
// 				.unwrap()
// 				.liquidity_gross,
// 		expandto18decimals(5).as_u128(),
// 	);
// }

// // Uniswap "limit orders"

// #[test]
// fn test_limitselling_basetopair_tick0thru1_lo() {
// 	let (mut pool, _, _, _) = mediumpool_initialized_zerotick_lo();

// 	// Value to emulate minted liquidity in Uniswap
// 	let liquiditytomint: u128 = 5981737760509663;

// 	let (minted_capital, _) = pool
// 		.mint_limit_order(
// 			id.clone(),
// 			-TICKSPACING_UNISWAP_MEDIUM,
// 			liquiditytomint,
// 			Side::Zero,
// 			|_| Ok::<(), ()>(()),
// 		)
// 		.unwrap();

// 	assert_eq!(minted_capital[Side::Zero], U256::from_dec_str("5981737760509663").unwrap());
// 	assert_eq!(minted_capital[!Side::Zero], U256::from_dec_str("0").unwrap());

// 	// somebody takes the limit order
// 	assert!(pool
// 		.swap::<Asset1ToAsset0>(U256::from_dec_str("2000000000000000000").unwrap())
// 		.is_ok());

// 	let (burnt, fees_owed) = pool
// 		.burn_limit_order(
// 			id.clone(),
// 			-TICKSPACING_UNISWAP_MEDIUM,
// 			liquiditytomint,
// 			Side::Zero,
// 		)
// 		.unwrap();
// 	assert_eq!(burnt[Side::Zero], U256::from_dec_str("0").unwrap());
// 	// For now just squaring the sqrt_price_at_tick
// 	let position_burnt = mul_div_floor(
// 		U256::from(liquiditytomint),
// 		PoolState::sqrt_price_at_tick(-TICKSPACING_UNISWAP_MEDIUM)
// 			.pow(U256::from_dec_str("2").unwrap()),
// 		U256::from(2).pow(U256::from_dec_str("96").unwrap()),
// 	);
// 	assert_eq!(burnt[!Side::Zero], position_burnt);

// 	// Original value: 18107525382602. Slightly different because the amount swapped in the
// 	// position/tick will be slightly different (tick will be crossed with slightly
// 	// different amounts)
// 	assert_eq!(fees_owed, 17891544354686);

// 	match pool.burn_limit_order(id, -TICKSPACING_UNISWAP_MEDIUM, 0, Side::Zero) {
// 		Err(PositionError::NonExistent) => {},
// 		_ => panic!("Expected NonExistent"),
// 	}
// }

// #[test]
// fn test_limitselling_basetopair_tick0thru1_poke_lo() {
// 	let (mut pool, _, _, _) = mediumpool_initialized_zerotick_lo();

// 	// Value to emulate minted liquidity in Uniswap
// 	let liquiditytomint: u128 = 5981737760509663;

// 	let (minted_capital, _) = pool
// 		.mint_limit_order(
// 			id.clone(),
// 			-TICKSPACING_UNISWAP_MEDIUM,
// 			liquiditytomint,
// 			Side::Zero,
// 			|_| Ok::<(), ()>(()),
// 		)
// 		.unwrap();

// 	assert_eq!(minted_capital[Side::Zero], U256::from_dec_str("5981737760509663").unwrap());
// 	assert_eq!(minted_capital[!Side::Zero], U256::from_dec_str("0").unwrap());

// 	// somebody takes the limit order
// 	assert!(pool
// 		.swap::<Asset1ToAsset0>(U256::from_dec_str("2000000000000000000").unwrap())
// 		.is_ok());

// 	// Poke
// 	let (burnt, fees_owed) = pool
// 		.burn_limit_order(id.clone(), -TICKSPACING_UNISWAP_MEDIUM, 0, Side::Zero)
// 		.unwrap();

// 	assert_eq!(burnt[Side::Zero], U256::from_dec_str("0").unwrap());
// 	assert_eq!(burnt[!Side::Zero], U256::from_dec_str("0").unwrap());
// 	assert_eq!(fees_owed, 17891544354686);

// 	let (burnt, fees_owed) = pool
// 		.burn_limit_order(id, -TICKSPACING_UNISWAP_MEDIUM, liquiditytomint, Side::Zero)
// 		.unwrap();
// 	assert_eq!(burnt[Side::Zero], U256::from_dec_str("0").unwrap());
// 	// For now just squaring the sqrt_price_at_tick
// 	let position_burnt = mul_div_floor(
// 		U256::from(liquiditytomint),
// 		PoolState::sqrt_price_at_tick(-TICKSPACING_UNISWAP_MEDIUM)
// 			.pow(U256::from_dec_str("2").unwrap()),
// 		U256::from(2).pow(U256::from_dec_str("96").unwrap()),
// 	);
// 	assert_eq!(burnt[!Side::Zero], position_burnt);
// 	assert_eq!(fees_owed, 0);
// }

// #[test]
// fn test_limitselling_pairtobase_tick1thru0_lo() {
// 	let (mut pool, _, _, _) = mediumpool_initialized_zerotick_lo();

// 	let liquiditytomint: u128 = 5981737760509663;

// 	let (minted_capital, _) = pool
// 		.mint_limit_order(
// 			id.clone(),
// 			TICKSPACING_UNISWAP_MEDIUM,
// 			liquiditytomint,
// 			Side::One,
// 			|_| Ok::<(), ()>(()),
// 		)
// 		.unwrap();

// 	assert_eq!(minted_capital[!Side::Zero], U256::from_dec_str("5981737760509663").unwrap());
// 	assert_eq!(minted_capital[Side::Zero], U256::from_dec_str("0").unwrap());

// 	// somebody takes the limit order
// 	assert!(pool
// 		.swap::<Asset0ToAsset1>(U256::from_dec_str("2000000000000000000").unwrap())
// 		.is_ok());

// 	let (burnt, fees_owed) = pool
// 		.burn_limit_order(
// 			id.clone(),
// 			TICKSPACING_UNISWAP_MEDIUM,
// 			expandto18decimals(1).as_u128(),
// 			Side::One,
// 		)
// 		.unwrap();
// 	assert_eq!(burnt[!Side::Zero], U256::from_dec_str("0").unwrap());
// 	// For now just squaring the sqrt_price_at_tick
// 	let position_burnt = mul_div_floor(
// 		U256::from(liquiditytomint),
// 		U256::from(2).pow(U256::from_dec_str("96").unwrap()),
// 		PoolState::sqrt_price_at_tick(-TICKSPACING_UNISWAP_MEDIUM)
// 			.pow(U256::from_dec_str("2").unwrap()),
// 	);
// 	assert_eq!(burnt[Side::Zero], position_burnt);

// 	// DIFF: position fully burnt
// 	assert_eq!(fees_owed, 18107525382602);

// 	match pool.burn_limit_order(id, TICKSPACING_UNISWAP_MEDIUM, 0, Side::One) {
// 		Err(PositionError::NonExistent) => {},
// 		_ => panic!("Expected NonExistent"),
// 	}
// }

// #[test]
// fn test_limitselling_pairtobase_tick1thru0_poke_lo() {
// 	let (mut pool, _, _, _) = mediumpool_initialized_zerotick_lo();

// 	let liquiditytomint: u128 = 5981737760509663;

// 	let (minted_capital, _) = pool
// 		.mint_limit_order(
// 			id.clone(),
// 			TICKSPACING_UNISWAP_MEDIUM,
// 			liquiditytomint,
// 			Side::One,
// 			|_| Ok::<(), ()>(()),
// 		)
// 		.unwrap();

// 	assert_eq!(minted_capital[!Side::Zero], U256::from_dec_str("5981737760509663").unwrap());
// 	assert_eq!(minted_capital[Side::Zero], U256::from_dec_str("0").unwrap());

// 	// somebody takes the limit order
// 	assert!(pool
// 		.swap::<Asset0ToAsset1>(U256::from_dec_str("2000000000000000000").unwrap())
// 		.is_ok());

// 	let (burnt, fees_owed) = pool
// 		.burn_limit_order(id.clone(), TICKSPACING_UNISWAP_MEDIUM, 0, Side::One)
// 		.unwrap();

// 	assert_eq!(burnt[!Side::Zero], U256::from_dec_str("0").unwrap());
// 	assert_eq!(burnt[Side::Zero], U256::from_dec_str("0").unwrap());
// 	assert_eq!(fees_owed, 18107525382602);

// 	let (burnt, fees_owed) = pool
// 		.burn_limit_order(
// 			id,
// 			TICKSPACING_UNISWAP_MEDIUM,
// 			expandto18decimals(1).as_u128(),
// 			Side::One,
// 		)
// 		.unwrap();
// 	assert_eq!(burnt[!Side::Zero], U256::from_dec_str("0").unwrap());
// 	// For now just squaring the sqrt_price_at_tick
// 	let position_burnt = mul_div_floor(
// 		U256::from(liquiditytomint),
// 		U256::from(2).pow(U256::from_dec_str("96").unwrap()),
// 		PoolState::sqrt_price_at_tick(-TICKSPACING_UNISWAP_MEDIUM)
// 			.pow(U256::from_dec_str("2").unwrap()),
// 	);
// 	assert_eq!(burnt[Side::Zero], position_burnt);

// 	// DIFF: position fully burnt
// 	assert_eq!(fees_owed, 0);
// }

// // #Collect

// #[test]
// fn test_multiplelps_lo() {
// 	let (mut pool, _, id) = lowpool_initialized_one();
// 	let id2: LiquidityProvider = LiquidityProvider::from([0xce; 32]);

// 	pool.mint_limit_order(
// 		id.clone(),
// 		TICKSPACING_UNISWAP_LOW,
// 		expandto18decimals(1).as_u128(),
// 		Side::One,
// 		|_| Ok::<(), ()>(()),
// 	)
// 	.unwrap();
// 	pool.mint_limit_order(
// 		id2.clone(),
// 		TICKSPACING_UNISWAP_LOW,
// 		expandto18decimals(2).as_u128(),
// 		Side::One,
// 		|_| Ok::<(), ()>(()),
// 	)
// 	.unwrap();

// 	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

// 	// poke positions
// 	let (_, fees_owed) =
// 		pool.burn_limit_order(id, TICKSPACING_UNISWAP_LOW, 0, Side::One).unwrap();

// 	// NOTE: Fee_owed value 1 unit different than Uniswap because uniswap requires 4
// 	// loops to do the swap instead of 1 causing the rounding to be different
// 	assert_eq!(fees_owed, 166666666666666_u128);

// 	let (_, fees_owed) = pool
// 		.burn_limit_order(id2, TICKSPACING_UNISWAP_LOW, 0, Side::One)
// 		.unwrap();
// 	// NOTE: Fee_owed value 1 unit different than Uniswap because uniswap requires 4
// 	// loops to do the swap instead of 1 causing the rounding to be different
// 	assert_eq!(fees_owed, 333333333333333_u128);
// }

// // type(uint128).max * 2**128 / 1e18
// // https://www.wolframalpha.com/input/?i=%282**128+-+1%29+*+2**128+%2F+1e18
// // U256::from_dec_str("115792089237316195423570985008687907852929702298719625575994"
// // ). unwr ap();

// // Works across large increases
// #[test]
// fn test_before_capbidn_lo() {
// 	let (mut pool, _, id) = lowpool_initialized_one();

// 	let initick = pool.current_tick;

// 	for asset in &[Side::Zero, Side::One] {
// 		pool.mint_limit_order(id.clone(), initick, expandto18decimals(1).as_u128(), *asset, |_| {
// 			Ok::<(), ()>(())
// 		})
// 		.unwrap();

// 		let liquidity_map = match *asset {
// 			Side::Zero => &mut pool.liquidity_map_base_lo,
// 			Side::One => &mut pool.liquidity_map_pair_lo,
// 		};

// 		let tickinfo_lo = liquidity_map.get_mut(&initick).unwrap();
// 		tickinfo_lo.fee_growth_inside =
// 			U256::from_dec_str("115792089237316195423570985008687907852929702298719625575994")
// 				.unwrap();

// 		let (burnt, fees_owed) = pool.burn_limit_order(id.clone(), initick, 0, *asset).unwrap();

// 		assert_eq!(burnt[*asset], U256::from_dec_str("0").unwrap());
// 		assert_eq!(burnt[!*asset], U256::from_dec_str("0").unwrap());

// 		assert_eq!(fees_owed, u128::MAX - 1);
// 	}
// }

// #[test]
// fn test_after_capbidn_lo() {
// 	let (mut pool, _, id) = lowpool_initialized_one();

// 	let initick = pool.current_tick;

// 	for asset in &[Side::Zero, Side::One] {
// 		pool.mint_limit_order(id.clone(), initick, expandto18decimals(1).as_u128(), *asset, |_| {
// 			Ok::<(), ()>(())
// 		})
// 		.unwrap();

// 		let liquidity_map = match *asset {
// 			Side::Zero => &mut pool.liquidity_map_base_lo,
// 			Side::One => &mut pool.liquidity_map_pair_lo,
// 		};

// 		let tickinfo_lo = liquidity_map.get_mut(&initick).unwrap();
// 		tickinfo_lo.fee_growth_inside =
// 			U256::from_dec_str("115792089237316195423570985008687907852929702298719625575995")
// 				.unwrap();

// 		let (burnt, fees_owed) = pool.burn_limit_order(id.clone(), initick, 0, *asset).unwrap();

// 		assert_eq!(burnt[*asset], U256::from_dec_str("0").unwrap());
// 		assert_eq!(burnt[!*asset], U256::from_dec_str("0").unwrap());

// 		assert_eq!(fees_owed, u128::MAX);
// 	}
// }

// #[test]
// fn test_wellafter_capbidn_lo() {
// 	let (mut pool, _, id) = lowpool_initialized_one();

// 	let initick = pool.current_tick;

// 	for asset in &[Side::Zero, Side::One] {
// 		pool.mint_limit_order(id.clone(), initick, expandto18decimals(1).as_u128(), *asset, |_| {
// 			Ok::<(), ()>(())
// 		})
// 		.unwrap();

// 		let liquidity_map = match *asset {
// 			Side::Zero => &mut pool.liquidity_map_base_lo,
// 			Side::One => &mut pool.liquidity_map_pair_lo,
// 		};

// 		let tickinfo_lo = liquidity_map.get_mut(&initick).unwrap();
// 		tickinfo_lo.fee_growth_inside = U256::MAX;

// 		let (burnt, fees_owed) = pool.burn_limit_order(id.clone(), initick, 0, *asset).unwrap();

// 		assert_eq!(burnt[*asset], U256::from_dec_str("0").unwrap());
// 		assert_eq!(burnt[!*asset], U256::from_dec_str("0").unwrap());

// 		assert_eq!(fees_owed, u128::MAX);
// 	}
// }

// // DIFF: pool.global_fee_growth won't overflow. We make it saturate.

// fn lowpool_initialized_setfees_lo() -> (PoolState, PoolAssetMap<AmountU256>, LiquidityProvider) {
// 	let (mut pool, mut minted_amounts_accum, id) = lowpool_initialized_one();
// 	let id2: LiquidityProvider = LiquidityProvider::from([0xce; 32]);

// 	let initick = pool.current_tick;

// 	// Mint mock positions to initialize tick
// 	pool.mint_limit_order(id2.clone(), initick, 1, Side::Zero, |_| Ok::<(), ()>(()))
// 		.unwrap();
// 	pool.mint_limit_order(id2, initick, 1, Side::One, |_| Ok::<(), ()>(()))
// 		.unwrap();

// 	// Set fee growth inside to max.
// 	pool.liquidity_map_base_lo.get_mut(&initick).unwrap().fee_growth_inside = U256::MAX;
// 	pool.liquidity_map_pair_lo.get_mut(&initick).unwrap().fee_growth_inside = U256::MAX;

// 	// Initialize positions with fee_growth_inside

// 	let (minted_capital, _) = pool
// 		.mint_limit_order(
// 			id.clone(),
// 			initick,
// 			expandto18decimals(10).as_u128(),
// 			Side::Zero,
// 			|_| Ok::<(), ()>(()),
// 		)
// 		.unwrap();

// 	minted_amounts_accum[Side::Zero] += minted_capital[Side::Zero];
// 	minted_amounts_accum[!Side::Zero] += minted_capital[!Side::Zero];

// 	let (minted_capital, _) = pool
// 		.mint_limit_order(
// 			id.clone(),
// 			initick,
// 			expandto18decimals(10).as_u128(),
// 			Side::One,
// 			|_| Ok::<(), ()>(()),
// 		)
// 		.unwrap();

// 	minted_amounts_accum[Side::Zero] += minted_capital[Side::Zero];
// 	minted_amounts_accum[!Side::Zero] += minted_capital[!Side::Zero];

// 	// Health check
// 	assert_eq!(minted_amounts_accum[Side::Zero], expandto18decimals(10));
// 	assert_eq!(minted_amounts_accum[!Side::Zero], expandto18decimals(10));

// 	(pool, minted_amounts_accum, id)
// }

// #[test]
// fn test_base_lo() {
// 	let (mut pool, _, id) = lowpool_initialized_setfees_lo();

// 	let initick = pool.current_tick;

// 	assert!(pool.swap::<Asset1ToAsset0>(expandto18decimals(1)).is_ok());

// 	let (_, fees_owed) = pool.burn_limit_order(id, initick, 0, Side::Zero).unwrap();

// 	// DIFF: no fees accrued - saturated
// 	assert_eq!(fees_owed, 0);
// }

// #[test]
// fn test_pair_lo() {
// 	let (mut pool, _, id) = lowpool_initialized_setfees_lo();

// 	let initick = pool.current_tick;

// 	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

// 	let (_, fees_owed) = pool.burn_limit_order(id, initick, 0, Side::One).unwrap();

// 	// DIFF: no fees accrued - saturated
// 	assert_eq!(fees_owed, 0);
// }

// // Skipped more fee protocol tests

// // Medium Fee, tickSpacing = 12, 1:1 price
// fn mediumpool_initialized_nomint() -> (PoolState, PoolAssetMap<AmountU256>, LiquidityProvider) {
// 	// fee_pips shall be one order of magnitude smaller than in the Uniswap pool (because
// 	// ONE_IN_HUNDREDTH_BIPS is /10)
// 	let pool = PoolState::new(3000, encodedprice1_1()).unwrap();
// 	let id: LiquidityProvider = LiquidityProvider::from([0xcf; 32]);
// 	let minted_amounts: PoolAssetMap<AmountU256> = Default::default();
// 	(pool, minted_amounts, id)
// }
// // DIFF: We have a tickspacing of 1, which means we will never have issues with it.
// #[test]
// fn test_tickspacing_lo() {
// 	let (mut pool, _, id) = mediumpool_initialized_nomint();

// 	for asset in &[Side::Zero, Side::One] {
// 		pool.mint_limit_order(id.clone(), -6, 1, *asset, |_| Ok::<(), ()>(())).unwrap();
// 		pool.mint_limit_order(id.clone(), 6, 1, *asset, |_| Ok::<(), ()>(())).unwrap();
// 		pool.mint_limit_order(id.clone(), -12, 1, *asset, |_| Ok::<(), ()>(())).unwrap();
// 		pool.mint_limit_order(id.clone(), 12, 1, *asset, |_| Ok::<(), ()>(())).unwrap();
// 		pool.mint_limit_order(id.clone(), -120, 1, *asset, |_| Ok::<(), ()>(()))
// 			.unwrap();
// 		pool.mint_limit_order(id.clone(), 120, 1, *asset, |_| Ok::<(), ()>(())).unwrap();
// 		pool.mint_limit_order(id.clone(), -144, 1, *asset, |_| Ok::<(), ()>(()))
// 			.unwrap();
// 		pool.mint_limit_order(id.clone(), 144, 1, *asset, |_| Ok::<(), ()>(())).unwrap();
// 	}
// }

// #[test]
// fn test_swapping_gaps_pairtobase_lo() {
// 	let (mut pool, _, id) = mediumpool_initialized_nomint();
// 	// Change pool current tick so it uses the correct LO orders
// 	pool.current_tick = 150000;
// 	let liquidity_amount = 36096898321357_u128;

// 	// Mint two orders and check that it uses the correct one.
// 	// 120192 being the closest tick to the price that is swapped at Uniswap test
// 	pool.mint_limit_order(id.clone(), 120192, liquidity_amount, Side::Zero, |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();
// 	pool.mint_limit_order(id.clone(), 121200, liquidity_amount, Side::Zero, |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();

// 	assert!(pool.swap::<Asset1ToAsset0>(expandto18decimals(1)).is_ok());

// 	// This order should not have been used

// 	let (returned_capital, fees_owed) = pool
// 		.burn_limit_order(id.clone(), 121200, liquidity_amount, Side::Zero)
// 		.unwrap();

// 	assert_eq!(returned_capital[Side::Zero].as_u128(), liquidity_amount);
// 	assert_eq!(returned_capital[!Side::Zero].as_u128(), 0);
// 	assert_eq!(fees_owed, 0);

// 	// Poke to get the fees
// 	let (returned_capital, fees_owed) =
// 		pool.burn_limit_order(id, 120192, 0, Side::Zero).unwrap();

// 	assert!(fees_owed > 0);

// 	// Slightly different amounts because of price difference
// 	// Orig value: 30027458295511
// 	assert_eq!(returned_capital[Side::Zero], U256::from_dec_str("30083999478255").unwrap());
// 	// Substracting fees
// 	// Orig value: 996999999999848369
// 	assert_eq!(
// 		returned_capital[!Side::Zero],
// 		U256::from_dec_str("996999999999682559").unwrap()
// 	);

// 	// Tick should not have changed
// 	assert_eq!(pool.current_tick, 150000)
// }

// #[test]
// fn test_swapping_gaps_basetopair_lo() {
// 	let (mut pool, _, id) = mediumpool_initialized_nomint();
// 	// Change pool current tick so it uses the correct LO orders
// 	pool.current_tick = 150000;
// 	let liquidity_amount = 36096898321357_u128;

// 	// Mint two orders and check that it uses the correct one.
// 	// 120192 being the closest tick to the price that is swapped at Uniswap test
// 	pool.mint_limit_order(id.clone(), 120192, liquidity_amount, Side::One, |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();
// 	pool.mint_limit_order(id.clone(), 121200, liquidity_amount, Side::One, |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();

// 	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

// 	// This order should not have been used

// 	let (returned_capital, fees_owed) = pool
// 		.burn_limit_order(id.clone(), 121200, liquidity_amount, Side::One)
// 		.unwrap();

// 	assert_eq!(returned_capital[!Side::Zero].as_u128(), liquidity_amount);
// 	assert_eq!(returned_capital[Side::Zero].as_u128(), 0);
// 	assert_eq!(fees_owed, 0);

// 	// Poke to get the fees
// 	let (returned_capital, fees_owed) =
// 		pool.burn_limit_order(id, 120192, 0, Side::One).unwrap();

// 	assert!(fees_owed > 0);

// 	// Slightly different amounts because of price difference
// 	// Orig value: 30027458295511
// 	assert_eq!(returned_capital[!Side::Zero], U256::from_dec_str("30083999478255").unwrap());
// 	// Substracting fees
// 	// Orig value: 996999999999848369
// 	assert_eq!(
// 		returned_capital[Side::Zero],
// 		U256::from_dec_str("996999999999682559").unwrap()
// 	);

// 	// Tick should not have changed
// 	assert_eq!(pool.current_tick, 150000)
// }

// ///////////////////////////////////////////////////////////
// ///             Extra limit order tests                ////
// ///////////////////////////////////////////////////////////

// ////// LO Testing utilities //////

// // This function will probably be implemented inside the AMM - most likely in a better
// // way, as squaring the sqrt price is not optimal.
// fn aux_get_price_at_tick(tick: Tick) -> PriceQ128F96 {
// 	PoolState::sqrt_price_at_tick(tick).pow(U256::from_dec_str("2").unwrap())
// }

// // Check partially swapped single limit order
// // Fully burn a partially swapped position
// fn check_swap_one_tick_exactin(
// 	pool: &mut PoolState,
// 	amount_swap_in: AmountU256,
// 	amount_swap_out: AmountU256,
// 	asset_limit_order: PoolSide,
// 	price_limit_order: PriceQ128F96,
// 	total_fee_paid: AmountU256,
// ) {
// 	let amount_minus_fees = mul_div_floor(
// 		amount_swap_in,
// 		U256::from(ONE_IN_HUNDREDTH_BIPS - pool.fee_100th_bips),
// 		U256::from(ONE_IN_HUNDREDTH_BIPS),
// 	); // This cannot overflow as we bound fee_100th_bips to <= ONE_IN_HUNDREDTH_BIPS/2

// 	assert_eq!(amount_swap_in - amount_minus_fees, total_fee_paid);

// 	let position_swapped =
// 		calculate_amount(amount_minus_fees, price_limit_order, asset_limit_order);
// 	assert_eq!(position_swapped, amount_swap_out);
// }

// #[allow(clippy::too_many_arguments)]
// // Fully burn a partially swapped position
// fn check_and_burn_limitorder_swap_one_tick_exactin(
// 	pool: &mut PoolState,
// 	id: LiquidityProvider,
// 	tick_limit_order: Tick,
// 	amount_swap_in: AmountU256,
// 	amount_swap_out: AmountU256,
// 	asset_limit_order: PoolSide,
// 	total_fee_paid: AmountU256,
// 	amount_to_burn: Liquidity,
// 	tick_to_be_cleared: bool,
// ) {
// 	let price_limit_order = aux_get_price_at_tick(tick_limit_order);

// 	check_swap_one_tick_exactin(
// 		pool,
// 		amount_swap_in,
// 		amount_swap_out,
// 		asset_limit_order,
// 		price_limit_order,
// 		total_fee_paid,
// 	);

// 	// Burnt Limit Order
// 	let (returned_capital, fees_owed) = pool
// 		.burn_limit_order(id, tick_limit_order, amount_to_burn, asset_limit_order)
// 		.unwrap();
// 	if amount_swap_in == U256::from(0) {
// 		// No swap happened
// 		assert_eq!(fees_owed, 0);
// 	} else {
// 		// Swap happened
// 		assert!(fees_owed > 0);
// 	}

// 	// This will be the sum of fees and swapped position
// 	let amount_swapped_in_minus_fees =
// 		calculate_amount(amount_swap_out, price_limit_order, !asset_limit_order);

// 	// These checks might be off due to rounding - to check
// 	assert_eq!(returned_capital[!asset_limit_order], amount_swap_out);
// 	assert_eq!(U256::from(fees_owed), amount_swap_in - amount_swapped_in_minus_fees);
// 	assert_eq!(total_fee_paid, U256::from(fees_owed));

// 	let tick_result = get_tickinfo_limit_orders(pool, asset_limit_order, &tick_limit_order);

// 	match tick_result {
// 		Some(_) => assert!(tick_to_be_cleared),
// 		None => assert!(!tick_to_be_cleared),
// 	}
// }

// fn calculate_amount(
// 	amount_asset_in: AmountU256,
// 	price_limit_order: PriceQ128F96,
// 	asset_out: PoolSide,
// ) -> AmountU256 {
// 	if asset_out == Side::Zero {
// 		mul_div_floor(amount_asset_in, U256::from(1) << 96u32, price_limit_order)
// 	} else {
// 		mul_div_floor(amount_asset_in, price_limit_order, U256::from(1) << 96u32)
// 	}
// }

// /////////////////////////////////////////////////////////////////////////////////
// ///////////////////////// Tests added for limit orders //////////////////////////
// /////////////////////////////////////////////////////////////////////////////////

// // Initial tick == -23028
// // Initially no LO
// fn mint_pool_no_lo() -> (PoolState, PoolAssetMap<AmountU256>, LiquidityProvider) {
// 	let pool = PoolState::new(300, encodedprice1_10()).unwrap();
// 	let id: LiquidityProvider = LiquidityProvider::from([0xcf; 32]);
// 	let minted_amounts: PoolAssetMap<AmountU256> = Default::default();

// 	(pool, minted_amounts, id)
// }
// // Skipped collect tests
// #[test]
// fn test_swap_asset0_to_asset1_partial_swap_lo() {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	partial_swap_lo(&mut pool, id, Side::Zero);
// }

// #[test]
// fn test_swap_asset1_to_asset0_partial_swap_lo() {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	partial_swap_lo(&mut pool, id, Side::One);
// }

// fn partial_swap_lo(
// 	pool: &mut PoolState,
// 	id: LiquidityProvider,
// 	asset_in: PoolSide,
// ) -> (Tick, U256, U256, U256, u128) {
// 	let ini_liquidity = pool.current_liquidity;
// 	let ini_tick = pool.current_tick;
// 	let ini_price = pool.current_sqrt_price;

// 	let tick_limit_order = if asset_in == Side::One {
// 		pool.current_tick - TICKSPACING_UNISWAP_MEDIUM * 10
// 	} else {
// 		pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 10
// 	};

// 	let liquidity_amount = expandto18decimals(1).as_u128();

// 	// Limit order should partially been swapped
// 	let price_limit_order = aux_get_price_at_tick(tick_limit_order);
// 	// Pool has been initialized at around 1 : 10
// 	let price_ini = aux_get_price_at_tick(ini_tick);

// 	if asset_in == Side::Zero {
// 		// Check that lo price is > than initial price
// 		assert!(price_limit_order > price_ini);
// 	} else {
// 		// Check that lo price is < than initial price
// 		assert!(price_limit_order < price_ini);
// 	}

// 	pool.mint_limit_order(id, tick_limit_order, liquidity_amount, !asset_in, |_| Ok::<(), ()>(()))
// 		.unwrap();

// 	let amount_to_swap = expandto18decimals(1) / 10;

// 	let (total_amount_out, total_fee_paid) = if asset_in == Side::Zero {
// 		pool.swap::<Asset0ToAsset1>(amount_to_swap).unwrap()
// 	} else {
// 		pool.swap::<Asset1ToAsset0>(amount_to_swap).unwrap()
// 	};
// 	// Check swap outcomes
// 	// Tick, sqrtPrice and liquidity haven't changed (range order pool)
// 	assert_eq!(pool.current_liquidity, ini_liquidity);
// 	assert_eq!(pool.current_tick, ini_tick);
// 	assert_eq!(pool.current_sqrt_price, ini_price);

// 	check_swap_one_tick_exactin(
// 		pool,
// 		amount_to_swap,
// 		total_amount_out,
// 		!asset_in,
// 		price_limit_order,
// 		total_fee_paid,
// 	);
// 	(tick_limit_order, amount_to_swap, total_amount_out, total_fee_paid, liquidity_amount)
// }

// #[test]
// fn test_swap_asset0_to_asset1_full_swap_lo() {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	full_swap_lo(&mut pool, id, Side::Zero);
// }

// #[test]
// fn test_swap_asset1_to_asset0_full_swap_lo() {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	full_swap_lo(&mut pool, id, Side::One);
// }

// fn full_swap_lo(
// 	pool: &mut PoolState,
// 	id: LiquidityProvider,
// 	asset_in: PoolSide,
// ) -> (Tick, Tick, U256, U256) {
// 	let id2: LiquidityProvider = LiquidityProvider::from([0xce; 32]);

// 	let ini_liquidity = pool.current_liquidity;
// 	let ini_tick = pool.current_tick;
// 	let ini_price = pool.current_sqrt_price;

// 	let (tick_limit_order_0, tick_limit_order_1) = if asset_in == Side::One {
// 		(
// 			pool.current_tick - TICKSPACING_UNISWAP_MEDIUM * 10,
// 			pool.current_tick - TICKSPACING_UNISWAP_MEDIUM * 2,
// 		)
// 	} else {
// 		(
// 			pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 10,
// 			pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 2,
// 		)
// 	};

// 	let liquidity_amount = expandto18decimals(1).as_u128();

// 	pool.mint_limit_order(id2.clone(), tick_limit_order_0, liquidity_amount, !asset_in, |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();

// 	pool.mint_limit_order(
// 		id,
// 		tick_limit_order_1,
// 		liquidity_amount,
// 		!asset_in,
// 		|_| Ok::<(), ()>(()),
// 	)
// 	.unwrap();

// 	let amount_to_swap = expandto18decimals(1) / 15;
// 	let (total_amount_out, total_fee_paid) = if asset_in == Side::Zero {
// 		pool.swap::<Asset0ToAsset1>(amount_to_swap).unwrap()
// 	} else {
// 		pool.swap::<Asset1ToAsset0>(amount_to_swap).unwrap()
// 	};

// 	// This should have partially swapped the limit order placed
// 	let price_limit_order_0 = aux_get_price_at_tick(tick_limit_order_0);
// 	let price_limit_order_1 = aux_get_price_at_tick(tick_limit_order_1);
// 	// Pool has been initialized at around 1 : 10
// 	let price_ini = aux_get_price_at_tick(ini_tick);

// 	if asset_in == Side::Zero {
// 		// Check that lo price is > than initial price
// 		assert!(price_limit_order_0 > price_ini);
// 		assert!(price_limit_order_0 > price_limit_order_1);
// 	} else {
// 		// Check that lo price is < than initial price
// 		assert!(price_limit_order_0 < price_ini);
// 		assert!(price_limit_order_0 < price_limit_order_1);
// 	}

// 	// Check swap outcomes
// 	// Tick, sqrtPrice and liquidity haven't changed (range order pool)
// 	assert_eq!(pool.current_liquidity, ini_liquidity);
// 	assert_eq!(pool.current_tick, ini_tick);
// 	assert_eq!(pool.current_sqrt_price, ini_price);

// 	let amount_minus_fees = mul_div_floor(
// 		amount_to_swap,
// 		U256::from(ONE_IN_HUNDREDTH_BIPS - pool.fee_100th_bips),
// 		U256::from(ONE_IN_HUNDREDTH_BIPS),
// 	);

// 	// Part will be swapped from tickLO and part from tickLO1. Price will be worse than if
// 	// it was fully swapped from tickLO but better than if it was fully swapped in tick LO1
// 	let amount_out_iff_limit_order_0 =
// 		calculate_amount(amount_minus_fees, price_limit_order_0, !asset_in);
// 	let amount_out_iff_limit_order_1 =
// 		calculate_amount(amount_minus_fees, price_limit_order_0, !asset_in);

// 	assert!(total_amount_out < amount_out_iff_limit_order_0);
// 	assert!(total_amount_out > amount_out_iff_limit_order_1);

// 	// Check LO position and tick
// 	match get_limit_order(pool, !asset_in, tick_limit_order_0, id2) {
// 		None => {},
// 		_ => panic!("Expected NonExistent Key"),
// 	}

// 	let tick_1 = get_tickinfo_limit_orders(pool, !asset_in, &tick_limit_order_1).unwrap();
// 	let liquidity_left = tick_1.liquidity_gross * (tick_1.one_minus_percswap).as_u128();

// 	assert_eq!(U256::from(liquidity_left), U256::from(liquidity_amount) * 2 - total_amount_out);
// 	(tick_limit_order_0, tick_limit_order_1, total_amount_out, total_fee_paid)
// }

// #[test]
// fn test_mint_worse_lo_asset0_for_asset1() {
// 	mint_worse_lo_swap(Side::Zero);
// }

// #[test]
// fn test_mint_worse_lo_asset1_for_asset0() {
// 	mint_worse_lo_swap(Side::One);
// }

// fn mint_worse_lo_swap(asset_in: PoolSide) {
// 	let (mut pool, _, id) = mint_pool_no_lo();

// 	let tick_to_mint =
// 		if asset_in == Side::One { pool.current_tick - 1 } else { pool.current_tick + 1 };

// 	pool.mint_limit_order(
// 		id.clone(),
// 		tick_to_mint,
// 		expandto18decimals(1).as_u128(),
// 		!asset_in,
// 		|_| Ok::<(), ()>(()),
// 	)
// 	.unwrap();

// 	partial_swap_lo(&mut pool, id.clone(), asset_in);

// 	// Check LO position and tick
// 	match get_limit_order(&pool, !asset_in, tick_to_mint, id) {
// 		None => panic!("Expected existant Key"),
// 		Some(limit_order) => {
// 			// Limit order shouldn't have been used
// 			assert_eq!(limit_order.liquidity, expandto18decimals(1).as_u128());
// 			assert_eq!(limit_order.last_one_minus_percswap, U256::from(1));
// 		},
// 	}
// 	match get_tickinfo_limit_orders(&pool, !asset_in, &tick_to_mint) {
// 		None => panic!("Expected existant Key"),
// 		Some(tick) => {
// 			// Tick should have been used
// 			assert_eq!(tick.liquidity_gross, expandto18decimals(1).as_u128());
// 			assert_eq!(tick.one_minus_percswap, U256::from(1));
// 		},
// 	}
// }

// #[test]
// fn test_multiple_positions_asset0_for_asset1() {
// 	multiple_positions(Side::Zero);
// }

// #[test]
// fn test_multiple_positions_asset1_for_asset0() {
// 	multiple_positions(Side::One);
// }

// fn multiple_positions(asset_in: PoolSide) {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	let id2: LiquidityProvider = LiquidityProvider::from([0xce; 32]);

// 	let tick_to_mint = if asset_in == Side::One {
// 		pool.current_tick - TICKSPACING_UNISWAP_MEDIUM * 10
// 	} else {
// 		pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 10
// 	};

// 	let initial_liquidity = expandto18decimals(1).as_u128();

// 	pool.mint_limit_order(id.clone(), tick_to_mint, initial_liquidity, !asset_in, |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();
// 	pool.mint_limit_order(id2.clone(), tick_to_mint, initial_liquidity, !asset_in, |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();

// 	// Check tick before swapping
// 	match get_tickinfo_limit_orders(&pool, !asset_in, &tick_to_mint) {
// 		None => panic!("Expected existant Key"),
// 		Some(tick) => {
// 			assert_eq!(tick.liquidity_gross, initial_liquidity * 2);
// 			assert_eq!(tick.one_minus_percswap, U256::from(1));
// 		},
// 	}

// 	let amount_to_swap = expandto18decimals(10);

// 	// To cross the first tick (=== first position tickL0) and part of the second (tickL01)
// 	let (total_amount_out, total_fee_paid) = if asset_in == Side::Zero {
// 		pool.swap::<Asset0ToAsset1>(amount_to_swap).unwrap()
// 	} else {
// 		pool.swap::<Asset1ToAsset0>(amount_to_swap).unwrap()
// 	};

// 	check_swap_one_tick_exactin(
// 		&mut pool,
// 		amount_to_swap,
// 		total_amount_out,
// 		asset_in,
// 		aux_get_price_at_tick(tick_to_mint),
// 		total_fee_paid,
// 	);

// 	// Check position and tick
// 	match get_limit_order(&pool, !asset_in, tick_to_mint, id) {
// 		None => panic!("Expected existant Key"),
// 		Some(limit_order) => {
// 			assert_eq!(limit_order.liquidity, expandto18decimals(1).as_u128());
// 			assert_eq!(limit_order.last_one_minus_percswap, U256::from(1));
// 		},
// 	}
// 	match get_limit_order(&pool, !asset_in, tick_to_mint, id2) {
// 		None => panic!("Expected existant Key"),
// 		Some(limit_order) => {
// 			assert_eq!(limit_order.liquidity, expandto18decimals(1).as_u128());
// 			assert_eq!(limit_order.last_one_minus_percswap, U256::from(1));
// 		},
// 	}

// 	match get_tickinfo_limit_orders(&pool, !asset_in, &tick_to_mint) {
// 		None => panic!("Expected existant Key"),
// 		Some(tick) => {
// 			assert_eq!(tick.liquidity_gross, initial_liquidity);
// 			assert!(tick.one_minus_percswap < U256::from(1));
// 		},
// 	}
// }

// // Skipped tests for ownerPositions - unclear how we will do that.
// // from test_chainflipPool.py line 2869 to 2955

// #[test]
// fn test_mint_partially_swapped_tick_asset0_for_asset1() {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	mint_partially_swapped_tick(&mut pool, id, Side::Zero);
// }

// #[test]
// fn test_mint_partially_swapped_tick_asset1_for_asset0() {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	mint_partially_swapped_tick(&mut pool, id, Side::One);
// }

// fn mint_partially_swapped_tick(
// 	pool: &mut PoolState,
// 	id: LiquidityProvider,
// 	asset_in: PoolSide,
// ) -> (Tick, U256, U256, U256, u128) {
// 	let id2 = LiquidityProvider::from([0xce; 32]);
// 	let (tick_to_mint, amount_swap_in, amount_swap_out, total_fee_paid, liquidity_amount) =
// 		partial_swap_lo(pool, id, asset_in);

// 	let tick_info = get_tickinfo_limit_orders(pool, !asset_in, &tick_to_mint).unwrap();

// 	let ini_liquidity_gross = tick_info.liquidity_gross;
// 	let ini_one_minus_perc_swapped = tick_info.one_minus_percswap;
// 	assert_eq!(ini_liquidity_gross, expandto18decimals(1).as_u128());
// 	assert!(ini_one_minus_perc_swapped < expandto18decimals(1));

// 	pool.mint_limit_order(
// 		id2.clone(),
// 		tick_to_mint,
// 		expandto18decimals(1).as_u128(),
// 		!asset_in,
// 		|_| Ok::<(), ()>(()),
// 	)
// 	.unwrap();
// 	let tick_info = get_tickinfo_limit_orders(pool, !asset_in, &tick_to_mint).unwrap();
// 	assert_eq!(tick_info.liquidity_gross, expandto18decimals(1).as_u128());
// 	assert_eq!(tick_info.one_minus_percswap, ini_one_minus_perc_swapped);
// 	assert_eq!(
// 		get_limit_order(pool, !asset_in, tick_to_mint, id2).unwrap().liquidity,
// 		expandto18decimals(1).as_u128()
// 	);
// 	(tick_to_mint, amount_swap_in, amount_swap_out, total_fee_paid, liquidity_amount)
// }

// #[test]
// fn test_mint_fully_swapped_tick_diff_account_asset0_for_asset1() {
// 	mint_fully_swapped_tick_diff_account(Side::Zero);
// }

// #[test]
// fn test_mint_fully_swapped_tick_diff_account_asset1_for_asset0() {
// 	mint_fully_swapped_tick_diff_account(Side::One);
// }

// fn mint_fully_swapped_tick_diff_account(asset_in: PoolSide) {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	let id3: LiquidityProvider = LiquidityProvider::from([0xcc; 32]);

// 	let (tick_limit_order_0, tick_limit_order_1, total_amount_out_0, total_fee_paid_0) =
// 		full_swap_lo(&mut pool, id, asset_in);

// 	// Check that tick_limit_order_1 is partially swapped and not removed
// 	let tick_info = get_tickinfo_limit_orders(&pool, !asset_in, &tick_limit_order_1).unwrap();
// 	assert!(tick_info.liquidity_gross > 0);
// 	assert!(tick_info.one_minus_percswap < U256::from(1));

// 	// Mint a position on top of the previous fully swapped position
// 	pool.mint_limit_order(
// 		id3,
// 		tick_limit_order_0,
// 		expandto18decimals(1).as_u128(),
// 		!asset_in,
// 		|_| Ok::<(), ()>(()),
// 	)
// 	.unwrap();

// 	let amount_to_swap = expandto18decimals(10);

// 	// Fully swap the newly minted position and part of the backup position
// 	// (tick_limit_order_1)
// 	let (total_amount_out_1, total_fee_paid_1) = if asset_in == Side::Zero {
// 		pool.swap::<Asset0ToAsset1>(amount_to_swap).unwrap()
// 	} else {
// 		pool.swap::<Asset1ToAsset0>(amount_to_swap).unwrap()
// 	};

// 	// Check that the results are the same as in the first swap
// 	assert_eq!(total_amount_out_0, total_amount_out_1);
// 	assert_eq!(total_fee_paid_0, total_fee_paid_1);
// }

// #[test]
// fn test_burn_position_minted_after_swap_asset0_for_asset1() {
// 	burn_position_minted_after_swap(Side::Zero);
// }

// #[test]
// fn test_burn_position_minted_after_swap_asset1_for_asset0() {
// 	burn_position_minted_after_swap(Side::One);
// }

// fn burn_position_minted_after_swap(asset_in: PoolSide) {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	let id2 = LiquidityProvider::from([0xcc; 32]);
// 	let (tick_to_mint, amount_swap_in, amount_swap_out, total_fee_paid, liquidity_position) =
// 		mint_partially_swapped_tick(&mut pool, id.clone(), asset_in);

// 	// Burn newly minted position
// 	check_and_burn_limitorder_swap_one_tick_exactin(
// 		&mut pool,
// 		id2,
// 		tick_to_mint,
// 		amount_swap_in,
// 		amount_swap_out,
// 		!asset_in,
// 		total_fee_paid,
// 		0,
// 		false,
// 	);

// 	// Check amounts and first position (partially swapped) - same check as in
// 	// test_swap0For1_partialSwap for the first minted position. Nothing should have changed
// 	// by minting and burning an extra position on top after the swap has taken place.
// 	check_and_burn_limitorder_swap_one_tick_exactin(
// 		&mut pool,
// 		id,
// 		tick_to_mint,
// 		amount_swap_in,
// 		amount_swap_out,
// 		!asset_in,
// 		U256::from(0), // fee collected in the poke
// 		liquidity_position,
// 		true,
// 	);
// }

// #[test]
// fn test_limitorder_currenttick() {
// 	let (mut pool, _, id) = mediumpool_initialized_nomint();

// 	let ini_tick = pool.current_tick;
// 	// Check no limit order exists
// 	assert!(pool.liquidity_map_base_lo.is_empty());
// 	assert!(pool.liquidity_map_pair_lo.is_empty());

// 	// Loop through the two assets, minting a position and check the tick info
// 	for asset in [Side::Zero, Side::One].iter() {
// 		pool.mint_limit_order(
// 			id.clone(),
// 			ini_tick,
// 			expandto18decimals(1).as_u128(),
// 			*asset,
// 			|_| Ok::<(), ()>(()),
// 		)
// 		.unwrap();
// 	}
// 	let tick_info_0 = get_tickinfo_limit_orders(&pool, Side::Zero, &ini_tick).unwrap();
// 	let tick_info_1 = get_tickinfo_limit_orders(&pool, Side::One, &ini_tick).unwrap();

// 	assert_eq!(tick_info_0.liquidity_gross, expandto18decimals(1).as_u128());
// 	assert_eq!(tick_info_0.one_minus_percswap, U256::from(1));
// 	assert_eq!(tick_info_1.liquidity_gross, expandto18decimals(1).as_u128());
// 	assert_eq!(tick_info_1.one_minus_percswap, U256::from(1));

// 	// Swap asset0 for asset1
// 	assert!(pool.swap::<Asset1ToAsset0>(expandto18decimals(1)).is_ok());

// 	assert_eq!(pool.current_tick, ini_tick);

// 	let tick_info_0 = get_tickinfo_limit_orders(&pool, Side::Zero, &ini_tick).unwrap();
// 	let tick_info_1 = get_tickinfo_limit_orders(&pool, Side::One, &ini_tick).unwrap();

// 	// Tick 1 not altered
// 	assert_eq!(tick_info_1.liquidity_gross, expandto18decimals(1).as_u128());
// 	assert_eq!(tick_info_1.one_minus_percswap, U256::from(1));

// 	// In one direction the limit order is taken
// 	assert_eq!(tick_info_0.liquidity_gross, expandto18decimals(1).as_u128());
// 	// Should be almost zero (not zero bc there are fees). Just checking that it has been
// 	// used.
// 	assert!(tick_info_0.one_minus_percswap < U256::from(1));

// 	// Swap asset1 for asset0
// 	assert!(pool.swap::<Asset1ToAsset0>(expandto18decimals(1)).is_ok());

// 	let tick_info_0 = get_tickinfo_limit_orders(&pool, Side::Zero, &ini_tick).unwrap();
// 	let tick_info_1 = get_tickinfo_limit_orders(&pool, Side::One, &ini_tick).unwrap();

// 	// In the other direction it is taken but not until the range orders don't change the
// 	// pool price
// 	assert_ne!(pool.current_tick, ini_tick);
// 	// Not ending at the border (MIN_TICK) but rather going to the next best LO tick - 1
// 	assert_eq!(pool.current_tick, ini_tick - 1);

// 	// Tick 0 not altered
// 	assert_eq!(tick_info_0.liquidity_gross, expandto18decimals(1).as_u128());
// 	assert!(tick_info_0.one_minus_percswap < U256::from(1));

// 	// Tick1 used
// 	assert_eq!(tick_info_1.liquidity_gross, expandto18decimals(1).as_u128());
// 	// Should be almost zero (not zero bc there are fees). Just checking that it has been
// 	// used.
// 	assert!(tick_info_1.one_minus_percswap < U256::from(1));
// }

// #[test]
// fn test_no_rangeorder_limitorder_worseprice_asset0() {
// 	no_rangeorder_limitorder_worseprice(Side::Zero);
// }
// #[test]
// fn test_no_rangeorder_limitorder_worseprice_asset1() {
// 	no_rangeorder_limitorder_worseprice(Side::One);
// }

// fn no_rangeorder_limitorder_worseprice(asset_in: PoolSide) {
// 	let (mut pool, _, id) = mediumpool_initialized_nomint();

// 	// Tick == 0
// 	let ini_tick = pool.current_tick;

// 	let tick_limit_order = if asset_in == Side::Zero {
// 		ini_tick - TICKSPACING_UNISWAP_MEDIUM * 10
// 	} else {
// 		ini_tick + TICKSPACING_UNISWAP_MEDIUM * 10
// 	};

// 	pool.mint_limit_order(id, tick_limit_order, expandto18decimals(1).as_u128(), !asset_in, |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();

// 	assert_ne!(pool.current_tick, ini_tick);

// 	// Order should be taken but not until the range orders don't change the pool price.
// 	// Not ending at the border but rather going to the next best LO tick.
// 	if asset_in == Side::One {
// 		assert_eq!(pool.current_tick, tick_limit_order);
// 	} else {
// 		assert_eq!(pool.current_tick, tick_limit_order - 1);
// 	}
// 	assert!(
// 		get_tickinfo_limit_orders(&pool, !asset_in, &ini_tick)
// 			.unwrap()
// 			.one_minus_percswap <
// 			U256::from(1)
// 	);
// 	assert_eq!(
// 		get_tickinfo_limit_orders(&pool, asset_in, &ini_tick)
// 			.unwrap()
// 			.one_minus_percswap,
// 		U256::from(1)
// 	);
// }

// #[test]
// fn test_burn_partiallyswapped_multiplesteps_asset0() {
// 	burn_partiallyswapped_multiplesteps(Side::Zero);
// }

// #[test]
// fn test_burn_partiallyswapped_multiplesteps_asset1() {
// 	burn_partiallyswapped_multiplesteps(Side::One);
// }

// fn burn_partiallyswapped_multiplesteps(asset_in: PoolSide) {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	let (tick_minted, _, _, _, _) = partial_swap_lo(&mut pool, id.clone(), asset_in);

// 	let mut pool_copy = pool.clone();

// 	let (returned_capital_0, fees_owed_0) = pool_copy
// 		.burn_limit_order(id.clone(), tick_minted, expandto18decimals(1).as_u128(), !asset_in)
// 		.unwrap();

// 	match pool_copy.burn_limit_order(id.clone(), tick_minted, 1, !asset_in) {
// 		Ok(_) => panic!("Should not be able to burn more than minted"),
// 		Err(PositionError::NonExistent) => {},
// 		Err(_) => panic!("Wrong error"),
// 	}

// 	// Arbitrary numbers (2,4)
// 	for i in 2..=4 {
// 		let mut pool_copy = pool.clone();
// 		let mut returned_capital_1_accum: PoolAssetMap<U256> = Default::default();
// 		let mut fees_owed_1_accum: u128 = Default::default();
// 		// Loop for value of i
// 		for _j in 0..i {
// 			// Fees owed will be returned in the first iteration
// 			let (returned_capital_1, fees_owed_1) = pool_copy
// 				.burn_limit_order(
// 					id.clone(),
// 					tick_minted,
// 					expandto18decimals(1).as_u128() / i,
// 					!asset_in,
// 				)
// 				.unwrap();
// 			returned_capital_1_accum[Side::Zero] += returned_capital_1[Side::Zero];
// 			returned_capital_1_accum[Side::One] += returned_capital_1[Side::One];
// 			fees_owed_1_accum += fees_owed_1;
// 		}
// 		match pool_copy.burn_limit_order(id.clone(), tick_minted, 1, !asset_in) {
// 			Ok(_) => panic!("Should not be able to burn more than minted"),
// 			Err(PositionError::NonExistent) => {},
// 			Err(_) => panic!("Wrong error"),
// 		}
// 		// There can be a small rounding error in favour of the pool when burning in
// 		// multiple steps
// 		assert_eq!(
// 			returned_capital_0[Side::Zero],
// 			returned_capital_1_accum[Side::Zero] + 1
// 		);
// 		assert_eq!(
// 			returned_capital_0[Side::One],
// 			returned_capital_1_accum[Side::One] + 1
// 		);
// 		assert_eq!(fees_owed_0, fees_owed_1_accum);
// 	}
// }

// #[test]
// fn test_mint_on_swapped_position_asset0() {
// 	mint_on_swapped_position(Side::Zero);
// }
// #[test]
// fn test_mint_on_swapped_position_asset1() {
// 	mint_on_swapped_position(Side::One);
// }

// fn mint_on_swapped_position(asset_in: PoolSide) {
// 	let (mut pool, _, id) = mint_pool_no_lo();
// 	let (tick_minted, _, _, _, liquidity_amount) = partial_swap_lo(&mut pool, id.clone(), asset_in);

// 	let mut pool_copy = pool.clone();

// 	// Amount of swapped tokens that should get burnt regardless of newly
// 	// minted orders on top
// 	let (returned_capital_0, fees_owed_0) = pool_copy
// 		.burn_limit_order(id.clone(), tick_minted, liquidity_amount, !asset_in)
// 		.unwrap();

// 	pool.mint_limit_order(id.clone(), tick_minted, liquidity_amount * 1000, !asset_in, |_| {
// 		Ok::<(), ()>(())
// 	})
// 	.unwrap();

// 	assert_eq!(
// 		get_limit_order(&pool, !asset_in, tick_minted, id.clone()).unwrap().liquidity,
// 		liquidity_amount * (1000 + 1)
// 	);
// 	// Burn to check if now the entire position gets swapped by the percentatge
// 	// swapped in the first swap
// 	let (returned_capital_1, fees_owed_1) = pool
// 		.burn_limit_order(id, tick_minted, liquidity_amount * (1001), !asset_in)
// 		.unwrap();

// 	assert_eq!(returned_capital_0[asset_in], returned_capital_1[asset_in]);
// 	assert_eq!(
// 		returned_capital_0[!asset_in],
// 		returned_capital_1[!asset_in] - liquidity_amount * 1000
// 	);
// 	assert_eq!(fees_owed_0, fees_owed_1);
// }