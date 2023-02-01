use super::*;

// UNISWAP TESTS => UniswapV3Pool.spec.ts

pub const TICKSPACING_UNISWAP_MEDIUM: Tick = 60;
pub const MIN_TICK_UNISWAP_MEDIUM: Tick = -887220;
pub const MAX_TICK_UNISWAP_MEDIUM: Tick = -MIN_TICK_UNISWAP_MEDIUM;

pub const INITIALIZE_LIQUIDITY_AMOUNT: u128 = 2000000000000000000u128;

pub const TICKSPACING_UNISWAP_LOW: Tick = 10;
pub const MIN_TICK_UNISWAP_LOW: Tick = -887220;
pub const MAX_TICK_UNISWAP_LOW: Tick = -MIN_TICK_UNISWAP_LOW;

// #Burn
fn pool_initialized_zerotick(
	mut pool: PoolState,
) -> (PoolState, PoolAssetMap<AmountU256>, AccountId) {
	let id: AccountId = AccountId::from([0xcf; 32]);
	let mut minted_capital = None;

	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM,
		INITIALIZE_LIQUIDITY_AMOUNT,
		|minted| {
			minted_capital.replace(minted);
			Ok::<(), ()>(())
		},
	)
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	(pool, minted_capital, id)
}

// Medium Fee, tickSpacing = 12, 1:1 price
fn mediumpool_initialized_zerotick() -> (PoolState, PoolAssetMap<AmountU256>, AccountId) {
	// fee_pips shall be one order of magnitude smaller than in the Uniswap pool (because
	// ONE_IN_HUNDREDTH_BIPS is /10)
	let pool = PoolState::new(3000, encodedprice1_1()).unwrap();
	pool_initialized_zerotick(pool)
}

fn checktickisclear(pool: &PoolState, tick: Tick) {
	match pool.liquidity_map.get(&tick) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}
}

fn checkticknotclear(pool: &PoolState, tick: Tick) {
	if pool.liquidity_map.get(&tick).is_none() {
		panic!("Expected Key")
	}
}

fn mint_pool() -> (PoolState, PoolAssetMap<AmountU256>, AccountId) {
	let mut pool =
		PoolState::new(3000, U256::from_dec_str("25054144837504793118650146401").unwrap()).unwrap(); // encodeSqrtPrice (1,10)
	let id: AccountId = AccountId::from([0xcf; 32]);
	const MINTED_LIQUIDITY: u128 = 3_161;
	let mut minted_capital = None;

	let _ = pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM,
		MINTED_LIQUIDITY,
		|minted| {
			minted_capital.replace(minted);
			Ok::<(), ()>(())
		},
	);
	let minted_capital = minted_capital.unwrap();

	(pool, minted_capital, id)
}

#[test]
fn test_initialize_failure() {
	match PoolState::new(1000, U256::from(1)) {
		Err(CreatePoolError::InvalidInitialPrice) => {},
		_ => panic!("Fees accrued are not zero"),
	}
}
#[test]
fn test_initialize_success() {
	let _ = PoolState::new(1000, MIN_SQRT_PRICE);
	let _ = PoolState::new(1000, MAX_SQRT_PRICE - 1);

	let pool =
		PoolState::new(1000, U256::from_dec_str("56022770974786143748341366784").unwrap()).unwrap();

	assert_eq!(
		pool.current_sqrt_price,
		U256::from_dec_str("56022770974786143748341366784").unwrap()
	);
	assert_eq!(pool.current_tick, -6_932);
}
#[test]
fn test_initialize_too_low() {
	match PoolState::new(1000, MIN_SQRT_PRICE - 1) {
		Err(CreatePoolError::InvalidInitialPrice) => {},
		_ => panic!("Fees accrued are not zero"),
	}
}

#[test]
fn test_initialize_too_high() {
	match PoolState::new(1000, MAX_SQRT_PRICE) {
		Err(CreatePoolError::InvalidInitialPrice) => {},
		_ => panic!("Fees accrued are not zero"),
	}
}

#[test]
fn test_initialize_too_high_2() {
	match PoolState::new(
		1000,
		U256::from_dec_str(
			"57896044618658097711785492504343953926634992332820282019728792003956564819968", /* 2**160-1 */
		)
		.unwrap(),
	) {
		Err(CreatePoolError::InvalidInitialPrice) => {},
		_ => panic!("Fees accrued are not zero"),
	}
}

// Minting

#[test]
fn test_mint_err() {
	let (mut pool, _, id) = mint_pool();
	assert!(pool.mint(id.clone(), 1, 0, 1, |_| Ok::<(), ()>(())).is_err());
	assert!((pool.mint(id.clone(), -887273, 0, 1, |_| Ok::<(), ()>(()))).is_err());
	assert!((pool.mint(id.clone(), 0, 887273, 1, |_| Ok::<(), ()>(()))).is_err());

	assert!((pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + 1,
		MAX_TICK_UNISWAP_MEDIUM - 1,
		MAX_TICK_GROSS_LIQUIDITY + 1,
		|_| Ok::<(), ()>(())
	))
	.is_err());

	assert!((pool.mint(
		id,
		MIN_TICK_UNISWAP_MEDIUM + 1,
		MAX_TICK_UNISWAP_MEDIUM - 1,
		MAX_TICK_GROSS_LIQUIDITY,
		|_| Ok::<(), ()>(())
	))
	.is_ok());
}

#[test]
fn test_mint_err_tickmax() {
	let (mut pool, _, id) = mint_pool();

	let (_, fees_owed) = pool
		.mint(id.clone(), MIN_TICK_UNISWAP_MEDIUM + 1, MAX_TICK_UNISWAP_MEDIUM - 1, 1000, |_| {
			Ok::<(), ()>(())
		})
		.unwrap();

	//assert_eq!(fees_owed.unwrap()[PoolSide::Asset0], 0);
	// assert_eq!(fees_owed.unwrap()[PoolSide::Asset1], 0);
	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}

	assert!((pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + 1,
		MAX_TICK_UNISWAP_MEDIUM - 1,
		MAX_TICK_GROSS_LIQUIDITY - 1000 + 1,
		|_| Ok::<(), ()>(())
	))
	.is_err());

	assert!((pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + 2,
		MAX_TICK_UNISWAP_MEDIUM - 1,
		MAX_TICK_GROSS_LIQUIDITY - 1000 + 1,
		|_| Ok::<(), ()>(())
	))
	.is_err());

	assert!((pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + 1,
		MAX_TICK_UNISWAP_MEDIUM - 2,
		MAX_TICK_GROSS_LIQUIDITY - 1000 + 1,
		|_| Ok::<(), ()>(())
	))
	.is_err());

	let (_, fees_owed) = pool
		.mint(
			id.clone(),
			MIN_TICK_UNISWAP_MEDIUM + 1,
			MAX_TICK_UNISWAP_MEDIUM - 1,
			MAX_TICK_GROSS_LIQUIDITY - 1000,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();
	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}

	// Different behaviour from Uniswap - does not revert when minting 0
	let (_, fees_owed) = pool
		.mint(id, MIN_TICK_UNISWAP_MEDIUM + 1, MAX_TICK_UNISWAP_MEDIUM - 1, 0, |_| Ok::<(), ()>(()))
		.unwrap();
	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}
}

// Success cases

#[test]
fn test_balances() {
	let (_, minted_capital, _) = mint_pool();
	// Check "balances"
	const INPUT_TICKER: PoolSide = PoolSide::Asset0;
	assert_eq!(minted_capital[INPUT_TICKER], U256::from(9_996));
	assert_eq!(minted_capital[!INPUT_TICKER], U256::from(1_000));
}

#[test]
fn test_initial_tick() {
	let (pool, _, _) = mint_pool();
	// Check current tick
	assert_eq!(pool.current_tick, -23_028);
}

#[test]
fn above_current_price() {
	let (mut pool, mut minted_capital_accum, id) = mint_pool();

	const MINTED_LIQUIDITY: u128 = 10_000;
	const INPUT_TICKER: PoolSide = PoolSide::Asset0;

	let mut minted_capital = None;
	let (_, fees_owed) = pool
		.mint(id, -22980, 0, MINTED_LIQUIDITY, |minted| {
			minted_capital.replace(minted);
			Ok::<(), ()>(())
		})
		.unwrap();
	let minted_capital = minted_capital.unwrap();

	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}

	assert_eq!(minted_capital[!INPUT_TICKER], U256::from(0));

	minted_capital_accum[INPUT_TICKER] += minted_capital[INPUT_TICKER];
	minted_capital_accum[!INPUT_TICKER] += minted_capital[!INPUT_TICKER];

	assert_eq!(minted_capital_accum[INPUT_TICKER], U256::from(9_996 + 21_549));
	assert_eq!(minted_capital_accum[!INPUT_TICKER], U256::from(1_000));
}

#[test]
fn test_maxtick_maxleverage() {
	let (mut pool, mut minted_capital_accum, id) = mint_pool();
	let mut minted_capital = None;
	let uniswap_max_tick = 887220;
	let uniswap_tickspacing = 60;
	pool.mint(
		id,
		uniswap_max_tick - uniswap_tickspacing, /* 60 == Uniswap's tickSpacing */
		uniswap_max_tick,
		5070602400912917605986812821504, /* 2**102 */
		|minted| {
			minted_capital.replace(minted);
			Ok::<(), ()>(())
		},
	)
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_capital_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	assert_eq!(minted_capital_accum[PoolSide::Asset0], U256::from(9_996 + 828_011_525));
	assert_eq!(minted_capital_accum[!PoolSide::Asset0], U256::from(1_000));
}

#[test]
fn test_maxtick() {
	let (mut pool, mut minted_capital_accum, id) = mint_pool();
	let mut minted_capital = None;
	pool.mint(id, -22980, 887220, 10000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_capital_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	assert_eq!(minted_capital_accum[PoolSide::Asset0], U256::from(9_996 + 31_549));
	assert_eq!(minted_capital_accum[!PoolSide::Asset0], U256::from(1_000));
}

#[test]
fn test_removing_works_0() {
	let (mut pool, _, id) = mint_pool();
	let mut minted_capital = None;
	pool.mint(id.clone(), -240, 0, 10000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();

	let (returned_capital, fees_owed) = pool.burn(id, -240, 0, 10000).unwrap();

	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(120));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));

	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
}

#[test]
fn test_removing_works_twosteps_0() {
	let (mut pool, _, id) = mint_pool();
	let mut minted_capital = None;
	pool.mint(id.clone(), -240, 0, 10000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();

	let (returned_capital_0, fees_owed_0) = pool.burn(id.clone(), -240, 0, 10000 / 2).unwrap();
	let (returned_capital_1, fees_owed_1) = pool.burn(id, -240, 0, 10000 / 2).unwrap();

	assert_eq!(returned_capital_0[PoolSide::Asset0], U256::from(60));
	assert_eq!(returned_capital_0[!PoolSide::Asset0], U256::from(0));
	assert_eq!(returned_capital_1[PoolSide::Asset0], U256::from(60));
	assert_eq!(returned_capital_1[!PoolSide::Asset0], U256::from(0));

	assert_eq!(fees_owed_0[PoolSide::Asset0], 0);
	assert_eq!(fees_owed_0[!PoolSide::Asset0], 0);
	assert_eq!(fees_owed_1[PoolSide::Asset0], 0);
	assert_eq!(fees_owed_1[!PoolSide::Asset0], 0);
}

#[test]
fn test_addliquidityto_liquiditygross() {
	let (mut pool, _, id) = mint_pool();
	let (_, fees_owed) = pool.mint(id.clone(), -240, 0, 100, |_| Ok::<(), ()>(())).unwrap();

	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}

	assert_eq!(pool.liquidity_map.get(&-240).unwrap().liquidity_gross, 100);
	assert_eq!(pool.liquidity_map.get(&0).unwrap().liquidity_gross, 100);
	assert!(!pool.liquidity_map.contains_key(&1));
	assert!(!pool.liquidity_map.contains_key(&2));

	let (_, fees_owed) = pool.mint(id.clone(), -240, 1, 150, |_| Ok::<(), ()>(())).unwrap();

	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}
	assert_eq!(pool.liquidity_map.get(&-240).unwrap().liquidity_gross, 250);
	assert_eq!(pool.liquidity_map.get(&0).unwrap().liquidity_gross, 100);
	assert_eq!(pool.liquidity_map.get(&1).unwrap().liquidity_gross, 150);
	assert!(!pool.liquidity_map.contains_key(&2));

	let (_, fees_owed) = pool.mint(id, 0, 2, 60, |_| Ok::<(), ()>(())).unwrap();

	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}
	assert_eq!(pool.liquidity_map.get(&-240).unwrap().liquidity_gross, 250);
	assert_eq!(pool.liquidity_map.get(&0).unwrap().liquidity_gross, 160);
	assert_eq!(pool.liquidity_map.get(&1).unwrap().liquidity_gross, 150);
	assert_eq!(pool.liquidity_map.get(&2).unwrap().liquidity_gross, 60);
}

#[test]
fn test_remove_liquidity_liquiditygross() {
	let (mut pool, _, id) = mint_pool();
	pool.mint(id.clone(), -240, 0, 100, |_| Ok::<(), ()>(())).unwrap();
	pool.mint(id.clone(), -240, 0, 40, |_| Ok::<(), ()>(())).unwrap();
	let (_, fees_owed) = pool.burn(id, -240, 0, 90).unwrap();
	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}
	assert_eq!(pool.liquidity_map.get(&-240).unwrap().liquidity_gross, 50);
	assert_eq!(pool.liquidity_map.get(&0).unwrap().liquidity_gross, 50);
}

#[test]
fn test_clearsticklower_ifpositionremoved() {
	let (mut pool, _, id) = mint_pool();
	pool.mint(id.clone(), -240, 0, 100, |_| Ok::<(), ()>(())).unwrap();
	let (_, fees_owed) = pool.burn(id, -240, 0, 100).unwrap();
	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}
	assert!(!pool.liquidity_map.contains_key(&-240));
}

#[test]
fn test_clearstickupper_ifpositionremoved() {
	let (mut pool, _, id) = mint_pool();
	pool.mint(id.clone(), -240, 0, 100, |_| Ok::<(), ()>(())).unwrap();
	pool.burn(id, -240, 0, 100).unwrap();
	assert!(!pool.liquidity_map.contains_key(&0));
}

#[test]
fn test_clears_onlyunused() {
	let (mut pool, _, id) = mint_pool();
	pool.mint(id.clone(), -240, 0, 100, |_| Ok::<(), ()>(())).unwrap();
	pool.mint(id.clone(), -60, 0, 250, |_| Ok::<(), ()>(())).unwrap();
	pool.burn(id, -240, 0, 100).unwrap();
	assert!(!pool.liquidity_map.contains_key(&-240));
	assert_eq!(pool.liquidity_map.get(&0).unwrap().liquidity_gross, 250);
	assert_eq!(
		pool.liquidity_map.get(&0).unwrap().fee_growth_outside[PoolSide::Asset0],
		U256::from(0)
	);
	assert_eq!(
		pool.liquidity_map.get(&0).unwrap().fee_growth_outside[!PoolSide::Asset0],
		U256::from(0)
	);
}

// Including current price

#[test]
fn test_price_within_range() {
	let (mut pool, minted_capital_accum, id) = mint_pool();
	let mut minted_capital = None;
	pool.mint(
		id,
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		100,
		|minted| {
			minted_capital.replace(minted);
			Ok::<(), ()>(())
		},
	)
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(317));
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(32));

	assert_eq!(
		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
		U256::from(9_996 + 317)
	);
	assert_eq!(
		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
		U256::from(1_000 + 32)
	);
}

#[test]
fn test_initializes_lowertick() {
	let (mut pool, _, id) = mint_pool();
	pool.mint(
		id,
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		100,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	assert_eq!(
		pool.liquidity_map
			.get(&(MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM))
			.unwrap()
			.liquidity_gross,
		100
	);
}

#[test]
fn test_initializes_uppertick() {
	let (mut pool, _, id) = mint_pool();
	pool.mint(
		id,
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		100,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	assert_eq!(
		pool.liquidity_map
			.get(&(MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM))
			.unwrap()
			.liquidity_gross,
		100
	);
}

#[test]
fn test_minmax_tick() {
	let (mut pool, minted_capital_accum, id) = mint_pool();
	let mut minted_capital = None;
	pool.mint(id, MIN_TICK_UNISWAP_MEDIUM, MAX_TICK_UNISWAP_MEDIUM, 10000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(31623));
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(3163));

	assert_eq!(
		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
		U256::from(9_996 + 31623)
	);
	assert_eq!(
		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
		U256::from(1_000 + 3163)
	);
}

#[test]
fn test_removing() {
	let (mut pool, _, id) = mint_pool();
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		100,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	let (amounts_owed, _) = pool
		.burn(
			id.clone(),
			MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
			MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
			100,
		)
		.unwrap();

	assert_eq!(amounts_owed[PoolSide::Asset0], U256::from(316));
	assert_eq!(amounts_owed[!PoolSide::Asset0], U256::from(31));

	// DIFF: Burn will have burnt the entire position so it will be deleted.
	match pool.burn(
		id,
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		1,
	) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}
}

// Below current price

#[test]
fn test_transfer_token1_only() {
	let (mut pool, minted_capital_accum, id) = mint_pool();
	let mut minted_capital = None;
	pool.mint(id, -46080, -23040, 10000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(2162));

	assert_eq!(
		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
		U256::from(9_996)
	);
	assert_eq!(
		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
		U256::from(1_000 + 2162)
	);
}

#[test]
fn test_mintick_maxleverage() {
	let (mut pool, minted_capital_accum, id) = mint_pool();
	let mut minted_capital = None;
	pool.mint(
		id,
		MIN_TICK_UNISWAP_MEDIUM,
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		5070602400912917605986812821504, /* 2**102 */
		|minted| {
			minted_capital.replace(minted);
			Ok::<(), ()>(())
		},
	)
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(828011520));

	assert_eq!(
		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
		U256::from(9_996)
	);
	assert_eq!(
		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
		U256::from(1_000 + 828011520)
	);
}

#[test]
fn test_mintick() {
	let (mut pool, minted_capital_accum, id) = mint_pool();
	let mut minted_capital = None;
	pool.mint(id, MIN_TICK_UNISWAP_MEDIUM, -23040, 10000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(3161));

	assert_eq!(
		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
		U256::from(9_996)
	);
	assert_eq!(
		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
		U256::from(1_000 + 3161)
	);
}

#[test]
fn test_removing_works_1() {
	let (mut pool, _, id) = mint_pool();
	let mut minted_capital = None;
	pool.mint(id.clone(), -46080, -46020, 10000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();

	let (returned_capital, fees_owed) = pool.burn(id.clone(), -46080, -46020, 10000).unwrap();

	// DIFF: Burn will have burnt the entire position so it will be deleted.
	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(3));

	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);

	match pool.burn(id, -46080, -46020, 1) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}
}

// NOTE: There is no implementation of protocol fees so we skip those tests

#[test]
fn test_poke_uninitialized_position() {
	let (mut pool, _, id) = mint_pool();
	pool.mint(
		AccountId::from([0xce; 32]),
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		expandto18decimals(1).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	let swap_input: u128 = expandto18decimals(1).as_u128();

	assert!(pool.swap::<Asset0ToAsset1>((swap_input / 10).into()).is_ok());
	assert!(pool.swap::<Asset1ToAsset0>((swap_input / 100).into()).is_ok());

	match pool.burn(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		0,
	) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}

	let (_, fees_owed) = pool
		.mint(
			id.clone(),
			MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
			MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
			1,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	match (fees_owed[PoolSide::Asset0], fees_owed[PoolSide::Asset1]) {
		(0, 0) => {},
		_ => panic!("Fees accrued are not zero"),
	}

	let tick = pool
		.positions
		.get(&(
			id.clone(),
			MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
			MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		))
		.unwrap();
	assert_eq!(tick.liquidity, 1);
	assert_eq!(
		tick.last_fee_growth_inside[PoolSide::Asset0],
		U256::from_dec_str("102084710076281216349243831104605583").unwrap()
	);
	assert_eq!(
		tick.last_fee_growth_inside[!PoolSide::Asset0],
		U256::from_dec_str("10208471007628121634924383110460558").unwrap()
	);
	// assert_eq!(tick.fees_owed[PoolSide::Asset0], 0);
	// assert_eq!(tick.fees_owed[!PoolSide::Asset0], 0);

	let (returned_capital, fees_owed) = pool
		.burn(
			id.clone(),
			MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
			MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
			1,
		)
		.unwrap();

	// DIFF: Burn will have burnt the entire position so it will be deleted.
	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);

	// This could be missing + fees_owed[PoolSide::Asset0]
	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(3));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));

	match pool.positions.get(&(
		id,
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
	)) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}
}

// Own test
#[test]
fn test_multiple_burns() {
	let (mut pool, _, _id) = mediumpool_initialized_zerotick();
	// some activity that would make the ticks non-zero
	pool.mint(
		AccountId::from([0xce; 32]),
		MIN_TICK_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM,
		expandto18decimals(1).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());
	assert!(pool.swap::<Asset1ToAsset0>(expandto18decimals(1)).is_ok());

	// Should be able to do only 1 burn (1000000000000000000 / 987654321000000000)

	pool.burn(
		AccountId::from([0xce; 32]),
		MIN_TICK_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM,
		987654321000000000,
	)
	.unwrap();

	match pool.burn(
		AccountId::from([0xce; 32]),
		MIN_TICK_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM,
		987654321000000000,
	) {
		Err(PositionError::PositionLacksLiquidity) => {},
		_ => panic!("Expected InsufficientLiquidity"),
	}
}

#[test]
fn test_notclearposition_ifnomoreliquidity() {
	let (mut pool, _, _id) = mediumpool_initialized_zerotick();
	// some activity that would make the ticks non-zero
	pool.mint(
		AccountId::from([0xce; 32]),
		MIN_TICK_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM,
		expandto18decimals(1).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());
	assert!(pool.swap::<Asset1ToAsset0>(expandto18decimals(1)).is_ok());

	// Add a poke to update the fee growth and check it's value
	let (returned_capital, fees_owed) = pool
		.burn(AccountId::from([0xce; 32]), MIN_TICK_UNISWAP_MEDIUM, MAX_TICK_UNISWAP_MEDIUM, 0)
		.unwrap();

	assert_ne!(fees_owed[PoolSide::Asset0], 0);
	assert_ne!(fees_owed[!PoolSide::Asset0], 0);
	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));

	let pos = pool
		.positions
		.get(&(AccountId::from([0xce; 32]), MIN_TICK_UNISWAP_MEDIUM, MAX_TICK_UNISWAP_MEDIUM))
		.unwrap();
	assert_eq!(
		pos.last_fee_growth_inside[PoolSide::Asset0],
		U256::from_dec_str("340282366920938463463374607431768211").unwrap()
	);
	assert_eq!(
		pos.last_fee_growth_inside[!PoolSide::Asset0],
		U256::from_dec_str("340282366920938463463374607431768211").unwrap()
	);

	let (returned_capital, fees_owed) = pool
		.burn(
			AccountId::from([0xce; 32]),
			MIN_TICK_UNISWAP_MEDIUM,
			MAX_TICK_UNISWAP_MEDIUM,
			expandto18decimals(1).as_u128(),
		)
		.unwrap();

	// DIFF: Burn will have burnt the entire position so it will be deleted.
	// Also, fees will already have been collected in the first burn.
	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);

	// This could be missing + fees_owed[PoolSide::Asset0]
	assert_ne!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_ne!(returned_capital[!PoolSide::Asset0], U256::from(0));

	match pool.positions.get(&(
		AccountId::from([0xce; 32]),
		MIN_TICK_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM,
	)) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}
}

#[test]
fn test_clearstick_iflastposition() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	// some activity that would make the ticks non-zero
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

	pool.burn(
		id,
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		1,
	)
	.unwrap();

	checktickisclear(&pool, MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM);
	checktickisclear(&pool, MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM);
}

#[test]
fn test_clearlower_ifupperused() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	// some activity that would make the ticks non-zero
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + 2 * TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

	pool.burn(
		id,
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		1,
	)
	.unwrap();

	checktickisclear(&pool, MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM);
	checkticknotclear(&pool, MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM);
}

#[test]
fn test_clearupper_iflowerused() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	// some activity that would make the ticks non-zero
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - 2 * TICKSPACING_UNISWAP_MEDIUM,
		1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

	pool.burn(
		id,
		MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
		MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
		1,
	)
	.unwrap();

	checkticknotclear(&pool, MIN_TICK_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM);
	checktickisclear(&pool, MAX_TICK_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM);
}

// Low Fee, tickSpacing = 10, 1:1 price
fn lowpool_initialized_zerotick() -> (PoolState, PoolAssetMap<AmountU256>, AccountId) {
	// Tickspacing
	let pool = PoolState::new(500, encodedprice1_1()).unwrap(); //	encodeSqrtPrice (1,1)
	pool_initialized_zerotick(pool)
}

#[test]
fn test_mint_rightofcurrentprice() {
	let (mut pool, _, id) = lowpool_initialized_zerotick();

	let liquiditybefore = pool.current_liquidity;

	let mut minted_capital = None;
	pool.mint(id, TICKSPACING_UNISWAP_LOW, 2 * TICKSPACING_UNISWAP_LOW, 1000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert!(pool.current_liquidity >= liquiditybefore);

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(1));
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(0));
}

#[test]
fn test_mint_leftofcurrentprice() {
	let (mut pool, _, id) = lowpool_initialized_zerotick();

	let liquiditybefore = pool.current_liquidity;

	let mut minted_capital = None;
	pool.mint(id, -2 * TICKSPACING_UNISWAP_LOW, -TICKSPACING_UNISWAP_LOW, 1000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert!(pool.current_liquidity >= liquiditybefore);

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(1));
}

#[test]
fn test_mint_withincurrentprice() {
	let (mut pool, _, id) = lowpool_initialized_zerotick();

	let liquiditybefore = pool.current_liquidity;

	let mut minted_capital = None;
	pool.mint(id, -TICKSPACING_UNISWAP_LOW, TICKSPACING_UNISWAP_LOW, 1000, |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert!(pool.current_liquidity >= liquiditybefore);

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(1));
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(1));
}

#[test]
fn test_cannotremove_morethanposition() {
	let (mut pool, _, id) = lowpool_initialized_zerotick();

	pool.mint(
		id.clone(),
		-TICKSPACING_UNISWAP_LOW,
		TICKSPACING_UNISWAP_LOW,
		expandto18decimals(1).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	match pool.burn(
		id,
		-TICKSPACING_UNISWAP_LOW,
		TICKSPACING_UNISWAP_LOW,
		expandto18decimals(1).as_u128() + 1,
	) {
		Err(PositionError::PositionLacksLiquidity) => {},
		_ => panic!("Should not be able to remove more than position"),
	}
}

#[test]
fn test_collectfees_withincurrentprice() {
	let (mut pool, _, id) = lowpool_initialized_zerotick();

	pool.mint(
		id.clone(),
		-TICKSPACING_UNISWAP_LOW * 100,
		TICKSPACING_UNISWAP_LOW * 100,
		expandto18decimals(100).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	let liquiditybefore = pool.current_liquidity;
	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

	assert!(pool.current_liquidity >= liquiditybefore);

	// Poke
	let (returned_capital, fees_owed) = pool
		.burn(id, -TICKSPACING_UNISWAP_LOW * 100, TICKSPACING_UNISWAP_LOW * 100, 0)
		.unwrap();

	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));

	assert!(fees_owed[PoolSide::Asset0] > 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
}

// Post initialize at medium fee

#[test]
fn test_initial_liquidity() {
	let (pool, _, _) = mediumpool_initialized_zerotick();
	assert_eq!(pool.current_liquidity, expandto18decimals(2).as_u128());
}

#[test]
fn test_returns_insupply_inrange() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	pool.mint(
		id,
		-TICKSPACING_UNISWAP_MEDIUM,
		TICKSPACING_UNISWAP_MEDIUM,
		expandto18decimals(3).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	assert_eq!(pool.current_liquidity, expandto18decimals(5).as_u128());
}

#[test]
fn test_excludes_supply_abovetick() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	pool.mint(
		id,
		TICKSPACING_UNISWAP_MEDIUM,
		2 * TICKSPACING_UNISWAP_MEDIUM,
		expandto18decimals(3).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	assert_eq!(pool.current_liquidity, expandto18decimals(2).as_u128());
}

#[test]
fn test_excludes_supply_belowtick() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	pool.mint(
		id,
		-2 * TICKSPACING_UNISWAP_MEDIUM,
		-TICKSPACING_UNISWAP_MEDIUM,
		expandto18decimals(3).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	assert_eq!(pool.current_liquidity, expandto18decimals(2).as_u128());
}

#[test]
fn test_updates_exiting() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	assert_eq!(pool.current_liquidity, expandto18decimals(2).as_u128());

	pool.mint(id, 0, TICKSPACING_UNISWAP_MEDIUM, expandto18decimals(1).as_u128(), |_| {
		Ok::<(), ()>(())
	})
	.unwrap();
	assert_eq!(pool.current_liquidity, expandto18decimals(3).as_u128());

	// swap toward the left (just enough for the tick transition function to trigger)
	assert!(pool.swap::<Asset0ToAsset1>((1).into()).is_ok());

	assert_eq!(pool.current_tick, -1);
	assert_eq!(pool.current_liquidity, expandto18decimals(2).as_u128());
}

#[test]
fn test_updates_entering() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	assert_eq!(pool.current_liquidity, expandto18decimals(2).as_u128());

	pool.mint(id, -TICKSPACING_UNISWAP_MEDIUM, 0, expandto18decimals(1).as_u128(), |_| {
		Ok::<(), ()>(())
	})
	.unwrap();
	assert_eq!(pool.current_liquidity, expandto18decimals(2).as_u128());

	// swap toward the left (just enough for the tick transition function to trigger)
	assert!(pool.swap::<Asset0ToAsset1>((1).into()).is_ok());

	assert_eq!(pool.current_tick, -1);
	assert_eq!(pool.current_liquidity, expandto18decimals(3).as_u128());
}

// Uniswap "limit orders"

#[test]
fn test_limitselling_asset_0_to_asset1_tick0thru1() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	let mut minted_capital = None;
	pool.mint(id.clone(), 0, 120, expandto18decimals(1).as_u128(), |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset1ToAsset0>(U256::from_dec_str("2000000000000000000").unwrap())
		.is_ok());

	let (burned, fees_owed) =
		pool.burn(id.clone(), 0, 120, expandto18decimals(1).as_u128()).unwrap();
	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("6017734268818165").unwrap());

	// DIFF: position fully burnt
	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 18107525382602);

	match pool.burn(id, 0, 120, 1) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}

	assert!(pool.current_tick > 120)
}

#[test]
fn test_limitselling_asset_0_to_asset_1_tick0thru1_poke() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	let mut minted_capital = None;
	pool.mint(id.clone(), 0, 120, expandto18decimals(1).as_u128(), |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset1ToAsset0>(U256::from_dec_str("2000000000000000000").unwrap())
		.is_ok());

	let (burned, fees_owed) = pool.burn(id.clone(), 0, 120, 0).unwrap();
	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// DIFF: position fully burnt
	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 18107525382602);

	let (burned, fees_owed) = pool.burn(id, 0, 120, expandto18decimals(1).as_u128()).unwrap();
	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("6017734268818165").unwrap());

	// DIFF: position fully burnt
	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);

	assert!(pool.current_tick > 120)
}

#[test]
fn test_limitselling_asset_1_to_asset_0_tick1thru0() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	let mut minted_capital = None;
	pool.mint(id.clone(), -120, 0, expandto18decimals(1).as_u128(), |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset0ToAsset1>(U256::from_dec_str("2000000000000000000").unwrap())
		.is_ok());

	let (burned, fees_owed) =
		pool.burn(id.clone(), -120, 0, expandto18decimals(1).as_u128()).unwrap();
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("6017734268818165").unwrap());

	// DIFF: position fully burnt
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
	assert_eq!(fees_owed[PoolSide::Asset0], 18107525382602);

	match pool.burn(id, -120, 0, 1) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}

	assert!(pool.current_tick < -120)
}

#[test]
fn test_limitselling_asset_1_to_asset_0_tick1thru0_poke() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	let mut minted_capital = None;
	pool.mint(id.clone(), -120, 0, expandto18decimals(1).as_u128(), |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset0ToAsset1>(U256::from_dec_str("2000000000000000000").unwrap())
		.is_ok());

	let (burned, fees_owed) = pool.burn(id.clone(), -120, 0, 0).unwrap();
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
	assert_eq!(fees_owed[PoolSide::Asset0], 18107525382602);

	let (burned, fees_owed) =
		pool.burn(id.clone(), -120, 0, expandto18decimals(1).as_u128()).unwrap();
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("6017734268818165").unwrap());

	// DIFF: position fully burnt
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
	assert_eq!(fees_owed[PoolSide::Asset0], 0);

	match pool.burn(id, -120, 0, 1) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}

	assert!(pool.current_tick < -120)
}

// #Collect

// Low Fee, tickSpacing = 10, 1:1 price
fn lowpool_initialized_one() -> (PoolState, PoolAssetMap<AmountU256>, AccountId) {
	let pool = PoolState::new(500, encodedprice1_1()).unwrap();
	let id: AccountId = AccountId::from([0xcf; 32]);
	let minted_amounts: PoolAssetMap<AmountU256> = Default::default();
	(pool, minted_amounts, id)
}

#[test]
fn test_multiplelps() {
	let (mut pool, _, id) = lowpool_initialized_one();

	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_LOW,
		MAX_TICK_UNISWAP_LOW,
		expandto18decimals(1).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_LOW + TICKSPACING_UNISWAP_LOW,
		MAX_TICK_UNISWAP_LOW - TICKSPACING_UNISWAP_LOW,
		2000000000000000000,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

	// poke positions
	let (burned, fees_owed) =
		pool.burn(id.clone(), MIN_TICK_UNISWAP_LOW, MAX_TICK_UNISWAP_LOW, 0).unwrap();

	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	// NOTE: Fee_owed value 1 unit different than Uniswap because uniswap requires 4 loops to do
	// the swap instead of 1 causing the rounding to be different
	assert_eq!(fees_owed[PoolSide::Asset0], 166666666666666u128);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);

	let (_, fees_owed) = pool
		.burn(
			id,
			MIN_TICK_UNISWAP_LOW + TICKSPACING_UNISWAP_LOW,
			MAX_TICK_UNISWAP_LOW - TICKSPACING_UNISWAP_LOW,
			0,
		)
		.unwrap();
	// NOTE: Fee_owed value 1 unit different than Uniswap because uniswap requires 4 loops to do
	// the swap instead of 1 causing the rounding to be different
	assert_eq!(fees_owed[PoolSide::Asset0], 333333333333333);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
}

// Works across large increases
#[test]
fn test_before_capbidn() {
	let (mut pool, _, id) = lowpool_initialized_one();
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_LOW,
		MAX_TICK_UNISWAP_LOW,
		expandto18decimals(1).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	pool.global_fee_growth[PoolSide::Asset0] =
		U256::from_dec_str("115792089237316195423570985008687907852929702298719625575994").unwrap();

	let (burned, fees_owed) = pool.burn(id, MIN_TICK_UNISWAP_LOW, MAX_TICK_UNISWAP_LOW, 0).unwrap();

	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	assert_eq!(fees_owed[PoolSide::Asset0], u128::MAX - 1);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
}

#[test]
fn test_after_capbidn() {
	let (mut pool, _, id) = lowpool_initialized_one();
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_LOW,
		MAX_TICK_UNISWAP_LOW,
		expandto18decimals(1).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	pool.global_fee_growth[PoolSide::Asset0] =
		U256::from_dec_str("115792089237316195423570985008687907852929702298719625575995").unwrap();

	let (burned, fees_owed) = pool.burn(id, MIN_TICK_UNISWAP_LOW, MAX_TICK_UNISWAP_LOW, 0).unwrap();

	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	assert_eq!(fees_owed[PoolSide::Asset0], u128::MAX);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
}

#[test]
fn test_wellafter_capbidn() {
	let (mut pool, _, id) = lowpool_initialized_one();
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_LOW,
		MAX_TICK_UNISWAP_LOW,
		expandto18decimals(1).as_u128(),
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	pool.global_fee_growth[PoolSide::Asset0] = U256::MAX;

	let (burned, fees_owed) = pool.burn(id, MIN_TICK_UNISWAP_LOW, MAX_TICK_UNISWAP_LOW, 0).unwrap();

	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	assert_eq!(fees_owed[PoolSide::Asset0], u128::MAX);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
}

// DIFF: pool.global_fee_growth won't overflow. We make it saturate.

fn lowpool_initialized_setfees() -> (PoolState, PoolAssetMap<AmountU256>, AccountId) {
	let (mut pool, mut minted_amounts_accum, id) = lowpool_initialized_one();
	pool.global_fee_growth[PoolSide::Asset0] = U256::MAX;
	pool.global_fee_growth[!PoolSide::Asset0] = U256::MAX;

	let mut minted_capital = None;
	pool.mint(
		id.clone(),
		MIN_TICK_UNISWAP_LOW,
		MAX_TICK_UNISWAP_LOW,
		expandto18decimals(10).as_u128(),
		|minted| {
			minted_capital.replace(minted);
			Ok::<(), ()>(())
		},
	)
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	minted_amounts_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_amounts_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	(pool, minted_amounts_accum, id)
}

#[test]
fn test_base() {
	let (mut pool, _, id) = lowpool_initialized_setfees();

	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

	assert_eq!(pool.global_fee_growth[PoolSide::Asset0], U256::MAX);
	assert_eq!(pool.global_fee_growth[!PoolSide::Asset0], U256::MAX);

	let (_, fees_owed) = pool.burn(id, MIN_TICK_UNISWAP_LOW, MAX_TICK_UNISWAP_LOW, 0).unwrap();

	// DIFF: no fees accrued
	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
}

#[test]
fn test_pair() {
	let (mut pool, _, id) = lowpool_initialized_setfees();

	assert!(pool.swap::<Asset1ToAsset0>(expandto18decimals(1)).is_ok());

	assert_eq!(pool.global_fee_growth[PoolSide::Asset0], U256::MAX);
	assert_eq!(pool.global_fee_growth[!PoolSide::Asset0], U256::MAX);

	let (_, fees_owed) = pool.burn(id, MIN_TICK_UNISWAP_LOW, MAX_TICK_UNISWAP_LOW, 0).unwrap();

	// DIFF: no fees accrued
	assert_eq!(fees_owed[PoolSide::Asset0], 0u128);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
}

///////////////////////////////////////////////////////////
///                  ADDED TESTS                       ////
///////////////////////////////////////////////////////////

// Add some more tests for fees_owed collecting

// Previous tests using mint as a poke and to collect fees.

#[test]
fn test_limit_selling_asset_0_to_asset_1_tick0thru1_mint() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	let mut minted_capital = None;
	pool.mint(id.clone(), 0, 120, expandto18decimals(1).as_u128(), |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset1ToAsset0>(U256::from_dec_str("2000000000000000000").unwrap())
		.is_ok());

	let (_, fees_owed) = pool.mint(id.clone(), 0, 120, 1, |_| Ok::<(), ()>(())).unwrap();

	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 18107525382602);

	let (_, fees_owed) = pool.mint(id.clone(), 0, 120, 1, |_| Ok::<(), ()>(())).unwrap();
	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);

	let (burned, fees_owed) = pool.burn(id, 0, 120, expandto18decimals(1).as_u128()).unwrap();
	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("6017734268818165").unwrap());

	// DIFF: position fully burnt
	assert_eq!(fees_owed[PoolSide::Asset0], 0);
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);

	assert!(pool.current_tick > 120)
}

#[test]
fn test_limit_selling_paior_tick1thru0_mint() {
	let (mut pool, _, id) = mediumpool_initialized_zerotick();
	let mut minted_capital = None;
	pool.mint(id.clone(), -120, 0, expandto18decimals(1).as_u128(), |minted| {
		minted_capital.replace(minted);
		Ok::<(), ()>(())
	})
	.unwrap();
	let minted_capital = minted_capital.unwrap();

	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset0ToAsset1>(U256::from_dec_str("2000000000000000000").unwrap())
		.is_ok());

	let (_, fees_owed) = pool.mint(id.clone(), -120, 0, 1, |_| Ok::<(), ()>(())).unwrap();

	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
	assert_eq!(fees_owed[PoolSide::Asset0], 18107525382602);

	let (_, fees_owed) = pool.mint(id.clone(), -120, 0, 1, |_| Ok::<(), ()>(())).unwrap();
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
	assert_eq!(fees_owed[PoolSide::Asset0], 0);

	let (burned, fees_owed) = pool.burn(id, -120, 0, expandto18decimals(1).as_u128()).unwrap();
	assert_eq!(burned[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burned[PoolSide::Asset0], U256::from_dec_str("6017734268818165").unwrap());

	// DIFF: position fully burnt
	assert_eq!(fees_owed[!PoolSide::Asset0], 0);
	assert_eq!(fees_owed[PoolSide::Asset0], 0);

	assert!(pool.current_tick < -120)
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

fn mint_pool_lo() -> (PoolState, PoolAssetMap<AmountU256>, AccountId, Tick, Tick) {
	let mut pool = PoolState::new(300, encodedprice1_10()).unwrap();
	let id: AccountId = AccountId::from([0xcf; 32]);
	const MINTED_LIQUIDITY: u128 = 3_161;

	let (mut minted_capital_accum, _) = pool
		.mint_limit_order(id.clone(), MIN_TICK_LO, MINTED_LIQUIDITY, PoolSide::Asset0, |_| {
			Ok::<(), ()>(())
		})
		.unwrap();

	let (minted_capital, _) = pool
		.mint_limit_order(id.clone(), MAX_TICK_LO, MINTED_LIQUIDITY, PoolSide::Asset1, |_| {
			Ok::<(), ()>(())
		})
		.unwrap();

	minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_capital_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	assert_eq!(pool.current_tick, -23028);

	// Closest ticks to the initialized pool tick with TICK_SPACING_MEDIUM_UNISWAP
	let close_initick_rdown: Tick = -23040;
	let close_initick_rup: Tick = -22980;

	(pool, minted_capital_accum, id, close_initick_rdown, close_initick_rup)
}

#[test]
fn test_trialmint_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();
	pool.mint_limit_order(id.clone(), MIN_TICK_LO, 3_161, PoolSide::Asset0, |_| Ok::<(), ()>(()))
		.unwrap();
	pool.mint_limit_order(id.clone(), MAX_TICK_LO, 3_161, PoolSide::Asset0, |_| Ok::<(), ()>(()))
		.unwrap();
	pool.mint_limit_order(id.clone(), MIN_TICK_LO, 3_161, PoolSide::Asset1, |_| Ok::<(), ()>(()))
		.unwrap();
	pool.mint_limit_order(id.clone(), MAX_TICK_LO, 3_161, PoolSide::Asset1, |_| Ok::<(), ()>(()))
		.unwrap();
}

// Minting

#[test]
fn test_mint_err_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		assert!((pool.mint_limit_order(id.clone(), -887273, 1, asset, |_| { Ok::<(), ()>(()) }))
			.is_err());
		assert!((pool
			.mint_limit_order(id.clone(), MIN_TICK_LO - 1, 1, asset, |_| { Ok::<(), ()>(()) }))
		.is_err());
		assert!((pool
			.mint_limit_order(id.clone(), MAX_TICK_LO + 1, 1, asset, |_| { Ok::<(), ()>(()) }))
		.is_err());

		assert!((pool.mint_limit_order(
			id.clone(),
			MIN_TICK_LO + 1,
			MAX_TICK_GROSS_LIQUIDITY + 1,
			asset,
			|_| { Ok::<(), ()>(()) }
		))
		.is_err());

		assert!((pool.mint_limit_order(
			id.clone(),
			MAX_TICK_LO - 1,
			MAX_TICK_GROSS_LIQUIDITY,
			asset,
			|_| { Ok::<(), ()>(()) }
		))
		.is_ok());
	}
}

#[test]
fn test_mint_err_tickmax_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		let (_, fees_owed) = pool
			.mint_limit_order(id.clone(), MAX_TICK_LO - 1, 1000, asset, |_| Ok::<(), ()>(()))
			.unwrap();

		match fees_owed {
			0 => {},
			_ => panic!("Fees accrued are not zero"),
		}

		assert!((pool.mint_limit_order(
			id.clone(),
			MAX_TICK_LO - 1,
			MAX_TICK_GROSS_LIQUIDITY - 1000 + 1,
			asset,
			|_| { Ok::<(), ()>(()) }
		))
		.is_err());

		assert!((pool.mint_limit_order(
			id.clone(),
			MIN_TICK_LO + 2,
			MAX_TICK_GROSS_LIQUIDITY - 1000 + 1,
			asset,
			|_| { Ok::<(), ()>(()) }
		))
		.is_err());

		assert!((pool.mint_limit_order(
			id.clone(),
			MAX_TICK_LO - 1,
			MAX_TICK_GROSS_LIQUIDITY - 1000 + 1,
			asset,
			|_| { Ok::<(), ()>(()) }
		))
		.is_err());

		assert!((pool.mint_limit_order(
			id.clone(),
			MAX_TICK_LO - 2,
			MAX_TICK_GROSS_LIQUIDITY - 1000 + 1,
			asset,
			|_| { Ok::<(), ()>(()) }
		))
		.is_err());

		assert!((pool.mint_limit_order(
			id.clone(),
			MIN_TICK_LO + 1,
			MAX_TICK_GROSS_LIQUIDITY - 1000 + 1,
			asset,
			|_| { Ok::<(), ()>(()) }
		))
		.is_err());

		let (_, fees_owed) = pool
			.mint_limit_order(
				id.clone(),
				MAX_TICK_LO - 1,
				MAX_TICK_GROSS_LIQUIDITY - 1000,
				asset,
				|_| Ok::<(), ()>(()),
			)
			.unwrap();
		match fees_owed {
			0 => {},
			_ => panic!("Fees accrued are not zero"),
		}

		// Different behaviour from Uniswap - does not revert when minting 0
		let (_, fees_owed) = pool
			.mint_limit_order(id.clone(), MAX_TICK_LO - 1, 0, asset, |_| Ok::<(), ()>(()))
			.unwrap();
		match fees_owed {
			0 => {},
			_ => panic!("Fees accrued are not zero"),
		}
	}
}

// Success cases

#[test]
fn test_balances_lo() {
	let (_, minted_capital, _, _, _) = mint_pool_lo();
	// Check "balances"
	const INPUT_TICKER: PoolSide = PoolSide::Asset0;
	assert_eq!(minted_capital[INPUT_TICKER], U256::from(3_161));
	assert_eq!(minted_capital[!INPUT_TICKER], U256::from(3_161));
}
#[test]
fn test_mint_one_side_lo() {
	let (mut pool, mut minted_capital_accum, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		// Adding one unit to this liquidity to make them different for testing purposes
		let (minted_capital, _) = pool
			.mint_limit_order(id.clone(), pool.current_tick, 3161, asset, |_| Ok::<(), ()>(()))
			.unwrap();

		minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
		minted_capital_accum[PoolSide::Asset1] += minted_capital[PoolSide::Asset1];

		assert_eq!(minted_capital_accum[asset], U256::from(3_161) * 2);
		assert_eq!(minted_capital_accum[!asset], U256::from(3_161));

		let (minted_capital, _) = pool
			.mint_limit_order(id.clone(), pool.current_tick, 3161, !asset, |_| Ok::<(), ()>(()))
			.unwrap();

		minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
		minted_capital_accum[PoolSide::Asset1] += minted_capital[PoolSide::Asset1];

		assert_eq!(minted_capital[asset], U256::from(3_161) * 2);
		assert_eq!(minted_capital[!asset], U256::from(3_161) * 2);
	}
}

#[test]
fn test_initial_tick_lo() {
	let (pool, _, _, _, _) = mint_pool_lo();
	// Check current tick - imit orders have not altered the tick
	assert_eq!(pool.current_tick, Tick::from(-23_028));
}

#[test]
// Above current price
fn test_transfer_onetoken_only_lo() {
	let (mut pool, mut minted_capital_accum, id, _, _) = mint_pool_lo();

	const MINTED_LIQUIDITY: u128 = 10_000;

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		let (minted_capital, fees_owed) = pool
			.mint_limit_order(id.clone(), -22980, MINTED_LIQUIDITY, asset, |_| Ok::<(), ()>(()))
			.unwrap();

		match fees_owed {
			0 => {},
			_ => panic!("Fees accrued are not zero"),
		}

		assert_eq!(minted_capital[!asset], U256::from(0));

		minted_capital_accum[asset] += minted_capital[asset];
		assert_eq!(minted_capital[!asset], U256::from(0));

		assert_eq!(minted_capital_accum[asset], U256::from(3_161 + 10000));
	}
}

#[test]

fn test_maxtick_maxleverage_lo() {
	let (mut pool, mut minted_capital_accum, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		let (minted_capital, _) = pool
			.mint_limit_order(
				id.clone(),
				MAX_TICK_LO,
				5070602400912917605986812821504 as u128,
				asset,
				|_| Ok::<(), ()>(()),
			)
			.unwrap();

		minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
		minted_capital_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

		assert_eq!(
			minted_capital_accum[asset],
			U256::from(3_161) + U256::from_dec_str("5070602400912917605986812821504").unwrap()
		);
		assert_eq!(minted_capital_accum[!asset], U256::from(3_161));
	}
}

#[test]
fn test_maxtick_lo() {
	let (mut pool, mut minted_capital_accum, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		let (minted_capital, _) = pool
			.mint_limit_order(id.clone(), MAX_TICK_LO, 10000, asset, |_| Ok::<(), ()>(()))
			.unwrap();

		minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
		minted_capital_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

		assert_eq!(minted_capital_accum[asset], U256::from(3_161 + 10_000));
		assert_eq!(minted_capital_accum[!asset], U256::from(3_161));
	}
}

#[test]
fn test_removing_works_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();
	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), -240, 10000, asset, |_| Ok::<(), ()>(()))
			.unwrap();
		pool.mint_limit_order(id.clone(), 0, 10000, asset, |_| Ok::<(), ()>(()))
			.unwrap();

		let (returned_capital, fees_owed) =
			pool.burn_limit_order(id.clone(), -240, 10000, asset).unwrap();

		assert_eq!(returned_capital[PoolSide::Asset0], U256::from(1000));
		assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));
		assert_eq!(fees_owed, 0);

		let (returned_capital, fees_owed) =
			pool.burn_limit_order(id.clone(), 0, 10000, asset).unwrap();

		assert_eq!(returned_capital[PoolSide::Asset0], U256::from(1000));
		assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));
		assert_eq!(fees_owed, 0);
	}
}

#[test]

fn test_removing_works_twosteps_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), -240, 10000, asset, |_| Ok::<(), ()>(()))
			.unwrap();

		for i in [-240, 0] {
			let (returned_capital_0, fees_owed_0) =
				pool.burn_limit_order(id.clone(), i, 10000 / 2, asset).unwrap();

			let (returned_capital_1, fees_owed_1) =
				pool.burn_limit_order(id.clone(), i, 10000 / 2, asset).unwrap();

			pool.mint_limit_order(id.clone(), i, 10000, asset, |_| Ok::<(), ()>(()))
				.unwrap();

			assert_eq!(returned_capital_0[PoolSide::Asset0], U256::from(60));
			assert_eq!(returned_capital_0[!PoolSide::Asset0], U256::from(0));
			assert_eq!(returned_capital_1[PoolSide::Asset0], U256::from(60));
			assert_eq!(returned_capital_1[!PoolSide::Asset0], U256::from(0));

			assert_eq!(fees_owed_0, 0);
			assert_eq!(fees_owed_1, 0);
		}
	}
}

fn get_tickinfo_limit_orders<'a>(
	pool: &'a PoolState,
	asset: PoolSide,
	tick: &'a Tick,
) -> Option<&'a TickInfoLimitOrder> {
	if asset == PoolSide::Asset0 {
		return pool.liquidity_map_base_lo.get(tick)
	} else {
		return pool.liquidity_map_pair_lo.get(tick)
	};
}

fn get_liquiditymap_lo<'a>(
	pool: &PoolState,
	asset: PoolSide,
) -> &BTreeMap<Tick, TickInfoLimitOrder> {
	if asset == PoolSide::Asset0 {
		return &pool.liquidity_map_base_lo
	} else {
		return &pool.liquidity_map_pair_lo
	};
}

#[test]
fn test_addliquidityto_liquiditygross_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		for i in [-240, 0] {
			let (_, fees_owed) =
				pool.mint_limit_order(id.clone(), i, 100, asset, |_| Ok::<(), ()>(())).unwrap();

			match fees_owed {
				0 => {},
				_ => panic!("Fees accrued are not zero"),
			}
			let tickinfo_lo = get_tickinfo_limit_orders(&pool, asset, &i).unwrap();
			assert_eq!(tickinfo_lo.liquidity_gross, 100);
			assert_eq!(tickinfo_lo.fee_growth_inside, U256::from(0));
			assert_eq!(tickinfo_lo.one_minus_percswap, U256::from(1));
			let liquidity_map_lo = get_liquiditymap_lo(&pool, asset);
			assert!(!liquidity_map_lo.contains_key(&1));
			assert!(!liquidity_map_lo.contains_key(&2));

			let (_, fees_owed) =
				pool.mint_limit_order(id.clone(), i, 150, asset, |_| Ok::<(), ()>(())).unwrap();
			match fees_owed {
				0 => {},
				_ => panic!("Fees accrued are not zero"),
			}
			let (_, fees_owed) = pool
				.mint_limit_order(id.clone(), i + 1, 150, asset, |_| Ok::<(), ()>(()))
				.unwrap();

			match fees_owed {
				0 => {},
				_ => panic!("Fees accrued are not zero"),
			}
			assert_eq!(get_tickinfo_limit_orders(&pool, asset, &i).unwrap().liquidity_gross, 250);
			assert_eq!(
				get_tickinfo_limit_orders(&pool, asset, &(i + 1)).unwrap().liquidity_gross,
				100
			);
			assert!(!get_liquiditymap_lo(&pool, asset).contains_key(&(i + 2)));

			let (_, fees_owed) = pool
				.mint_limit_order(id.clone(), i + 2, 60, asset, |_| Ok::<(), ()>(()))
				.unwrap();
			match fees_owed {
				0 => {},
				_ => panic!("Fees accrued are not zero"),
			}

			// Check complete final state
			let tickinfo_lo = get_tickinfo_limit_orders(&pool, asset, &i).unwrap();
			assert_eq!(tickinfo_lo.liquidity_gross, 250);
			assert_eq!(tickinfo_lo.fee_growth_inside, U256::from(0));
			assert_eq!(tickinfo_lo.one_minus_percswap, U256::from(1));
			let aux_tick = i + 1;
			let tickinfo_lo = get_tickinfo_limit_orders(&pool, asset, &aux_tick).unwrap();
			assert_eq!(tickinfo_lo.liquidity_gross, 100);
			assert_eq!(tickinfo_lo.fee_growth_inside, U256::from(0));
			assert_eq!(tickinfo_lo.one_minus_percswap, U256::from(1));
			let aux_tick = i + 2;
			let tickinfo_lo = get_tickinfo_limit_orders(&pool, asset, &aux_tick).unwrap();
			assert_eq!(tickinfo_lo.liquidity_gross, 60);
			assert_eq!(tickinfo_lo.fee_growth_inside, U256::from(0));
			assert_eq!(tickinfo_lo.one_minus_percswap, U256::from(1));
		}
	}
}

#[test]
fn test_remove_liquidity_liquiditygross_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), -240, 100, asset, |_| Ok::<(), ()>(()))
			.unwrap();
		pool.mint_limit_order(id.clone(), 0, 40, asset, |_| Ok::<(), ()>(())).unwrap();

		assert_eq!(get_tickinfo_limit_orders(&pool, asset, &240).unwrap().liquidity_gross, 100);
		assert_eq!(get_tickinfo_limit_orders(&pool, asset, &0).unwrap().liquidity_gross, 40);

		let (burnt, _) = pool.burn_limit_order(id.clone(), -240, 90, asset).unwrap();

		let tickinfo = get_tickinfo_limit_orders(&pool, asset, &240).unwrap();
		assert_eq!(tickinfo.liquidity_gross, 10);
		assert_eq!(tickinfo.fee_growth_inside, U256::from(0));
		assert_eq!(tickinfo.one_minus_percswap, U256::from(1));
		assert_eq!(burnt[asset], U256::from(90));
		assert_eq!(burnt[!asset], U256::from(0));

		let (burnt, _) = pool.burn_limit_order(id.clone(), 90, 30, asset).unwrap();
		let tickinfo = get_tickinfo_limit_orders(&pool, asset, &0).unwrap();
		assert_eq!(tickinfo.liquidity_gross, 10);
		assert_eq!(tickinfo.fee_growth_inside, U256::from(0));
		assert_eq!(tickinfo.one_minus_percswap, U256::from(1));
		assert_eq!(burnt[asset], U256::from(30));
		assert_eq!(burnt[!asset], U256::from(0));
	}
}

#[test]
fn test_clearstick_ifpositionremoved_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), -240, 100, asset, |_| Ok::<(), ()>(()))
			.unwrap();
		pool.mint_limit_order(id.clone(), 0, 100, asset, |_| Ok::<(), ()>(())).unwrap();

		pool.burn_limit_order(id.clone(), -240, 100, asset).unwrap();
		let liquidity_map_lo = get_liquiditymap_lo(&pool, asset);
		assert!(!liquidity_map_lo.contains_key(&-240));
		assert!(liquidity_map_lo.contains_key(&0));

		pool.burn_limit_order(id.clone(), 0, 100, asset).unwrap();
		let liquidity_map_lo = get_liquiditymap_lo(&pool, asset);
		assert!(!liquidity_map_lo.contains_key(&-240));
		assert!(!liquidity_map_lo.contains_key(&0));
	}
}

#[test]
fn test_clears_onlyunused_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();
	let id2: AccountId = AccountId::from([0xce; 32]);

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), -240, 100, asset, |_| Ok::<(), ()>(()))
			.unwrap();
		pool.mint_limit_order(id.clone(), 0, 100, asset, |_| Ok::<(), ()>(())).unwrap();
		pool.mint_limit_order(id2.clone(), -1, 250, asset, |_| Ok::<(), ()>(()))
			.unwrap();
		pool.mint_limit_order(id2.clone(), 0, 250, asset, |_| Ok::<(), ()>(())).unwrap();

		let liquidity_map_lo = get_liquiditymap_lo(&pool, asset);
		assert!(liquidity_map_lo.contains_key(&-240));
		assert!(liquidity_map_lo.contains_key(&0));
		assert!(liquidity_map_lo.contains_key(&1));

		pool.burn_limit_order(id.clone(), -240, 100, asset).unwrap();
		pool.burn_limit_order(id.clone(), 0, 100, asset).unwrap();

		let liquidity_map_lo = get_liquiditymap_lo(&pool, asset);
		assert!(!liquidity_map_lo.contains_key(&-240));
		let tickinfo_lo = get_tickinfo_limit_orders(&pool, asset, &-1).unwrap();
		assert_eq!(tickinfo_lo.liquidity_gross, 250);
		assert_eq!(tickinfo_lo.fee_growth_inside, U256::from(0));
		assert_eq!(tickinfo_lo.one_minus_percswap, U256::from(1));
	}
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

// 	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(317));
// 	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(32));

// 	assert_eq!(
// 		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
// 		U256::from(9_996 + 317)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
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

// 	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(31623));
// 	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(3163));

// 	assert_eq!(
// 		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
// 		U256::from(9_996 + 31623)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
// 		U256::from(1_000 + 3163)
// 	);
// }

#[test]
fn removing_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(
			id.clone(),
			MIN_TICK_LO_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
			100,
			asset,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();
		pool.mint_limit_order(
			id.clone(),
			MAX_TICK_LO_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
			101,
			asset,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

		let (amounts_owed, _) = pool
			.burn_limit_order(
				id.clone(),
				MIN_TICK_LO_UNISWAP_MEDIUM + TICKSPACING_UNISWAP_MEDIUM,
				100,
				asset,
			)
			.unwrap();

		assert_eq!(amounts_owed[asset], U256::from(100));
		assert_eq!(amounts_owed[asset], U256::from(0));

		let (amounts_owed, _) = pool
			.burn_limit_order(
				id.clone(),
				MAX_TICK_LO_UNISWAP_MEDIUM - TICKSPACING_UNISWAP_MEDIUM,
				101,
				asset,
			)
			.unwrap();

		assert_eq!(amounts_owed[asset], U256::from(101));
		assert_eq!(amounts_owed[asset], U256::from(0));
	}
}

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

// 	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(0));
// 	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(2162));

// 	assert_eq!(
// 		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
// 		U256::from(9_996)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
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

// 	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(0));
// 	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(828011520));

// 	assert_eq!(
// 		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
// 		U256::from(9_996)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
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

// 	assert_eq!(minted_capital[PoolSide::Asset0], U256::from(0));
// 	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from(3161));

// 	assert_eq!(
// 		minted_capital_accum[PoolSide::Asset0] + minted_capital[PoolSide::Asset0],
// 		U256::from(9_996)
// 	);
// 	assert_eq!(
// 		minted_capital_accum[!PoolSide::Asset0] + minted_capital[!PoolSide::Asset0],
// 		U256::from(1_000 + 3161)
// 	);
// }

#[test]
fn removing_works_1_lo() {
	let (mut pool, _, id, _, _) = mint_pool_lo();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), -46080, 10000, asset, |_| Ok::<(), ()>(()))
			.unwrap();
		pool.mint_limit_order(id.clone(), -46020, 10001, asset, |_| Ok::<(), ()>(()))
			.unwrap();

		let (returned_capital, fees_owed) =
			pool.burn_limit_order(id.clone(), -46080, 10000, asset).unwrap();

		// DIFF: Burn will have burnt the entire position so it will be deleted.
		assert_eq!(returned_capital[asset], U256::from(10000));
		assert_eq!(returned_capital[!asset], U256::from(0));

		assert_eq!(fees_owed, 0);

		match pool.burn_limit_order(id.clone(), -46080, 1, asset) {
			Err(PositionError::NonExistent) => {},
			_ => panic!("Expected NonExistent"),
		}
		let (returned_capital, fees_owed) =
			pool.burn_limit_order(id.clone(), -46020, 10001, asset).unwrap();

		// DIFF: Burn will have burnt the entire position so it will be deleted.
		assert_eq!(returned_capital[asset], U256::from(10001));
		assert_eq!(returned_capital[!asset], U256::from(0));

		assert_eq!(fees_owed, 0);

		match pool.burn_limit_order(id.clone(), -46080, 1, asset) {
			Err(PositionError::NonExistent) => {},
			_ => panic!("Expected NonExistent"),
		}
	}
}

fn get_limit_order(
	pool: &PoolState,
	asset: PoolSide,
	tick: Tick,
	id: AccountId,
) -> Option<&LimitOrders> {
	// TODO: update - probably will be pool.limit_orders.get(&(*id, *tick, asset))
	return pool.limit_orders[asset].get(&(id.clone(), tick))
}
// NOTE: There is no implementation of protocol fees so we skip those tests

#[test]
fn poke_uninitialized_position_lo() {
	let (mut pool, _, id, initick_rdown, initick_rup) = mint_pool_lo();

	let id2: AccountId = AccountId::from([0xce; 32]);

	pool.mint_limit_order(
		id2.clone(),
		initick_rdown,
		expandto18decimals(1).as_u128(),
		PoolSide::Asset0,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	pool.mint_limit_order(
		id2.clone(),
		initick_rup,
		expandto18decimals(1).as_u128(),
		PoolSide::Asset1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	let swap_input_amount: u128 = expandto18decimals(1).as_u128();

	assert!(pool.swap::<Asset0ToAsset1>((swap_input_amount / 10).into()).is_ok());
	assert!(pool.swap::<Asset1ToAsset0>((swap_input_amount / 100).into()).is_ok());

	// Assumption that swapping will remove the positions that are fully swapped

	match pool.burn_limit_order(id.clone(), initick_rdown, 0, PoolSide::Asset0) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}
	match pool.burn_limit_order(id.clone(), initick_rup, 0, PoolSide::Asset1) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}

	let (_, fees_owed_0) = pool
		.mint_limit_order(id.clone(), initick_rdown, 1, PoolSide::Asset0, |_| Ok::<(), ()>(()))
		.unwrap();
	let (_, fees_owed_1) = pool
		.mint_limit_order(id.clone(), initick_rup, 1, PoolSide::Asset1, |_| Ok::<(), ()>(()))
		.unwrap();

	let pos0 = get_limit_order(&pool, PoolSide::Asset0, initick_rdown, id.clone()).unwrap();
	let pos1 = get_limit_order(&pool, PoolSide::Asset1, initick_rup, id.clone()).unwrap();

	assert_eq!(pos0.liquidity, 1);
	// Orig value: 102084710076281216349243831104605583
	assert_eq!(
		pos0.last_fee_growth_inside,
		U256::from_dec_str("102084710076282900168480065983384316").unwrap()
	);
	// Orig value: 10208471007628121634924383110460558
	assert_eq!(
		pos1.last_fee_growth_inside,
		U256::from_dec_str("10208471007628121634924383110460558").unwrap()
	);
	assert_eq!(fees_owed_0, 0);
	assert_eq!(fees_owed_1, 0);

	// Poke to update feeGrowthInsideLastX128

	pool.burn_limit_order(id.clone(), initick_rdown, 0, PoolSide::Asset0).unwrap();
	pool.burn_limit_order(id.clone(), initick_rup, 0, PoolSide::Asset1).unwrap();

	let pos0 = get_limit_order(&pool, PoolSide::Asset0, initick_rdown, id.clone()).unwrap();
	let pos1 = get_limit_order(&pool, PoolSide::Asset1, initick_rup, id.clone()).unwrap();

	assert_eq!(pos0.liquidity, 1);
	assert_eq!(
		pos0.last_fee_growth_inside,
		U256::from_dec_str("102084710076282900168480065983384316").unwrap()
	);
	assert_eq!(
		pos1.last_fee_growth_inside,
		U256::from_dec_str("10208471007628121634924383110460558").unwrap(),
	);

	let (returned_capital, fees_owed) =
		pool.burn_limit_order(id.clone(), initick_rdown, 0, PoolSide::Asset0).unwrap();

	// DIFF: Burn will have burnt the entire position so it will be deleted.
	assert_eq!(fees_owed, 0);

	// This could be missing + fees_owed[PoolSide::Asset0]
	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(1));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));

	let (returned_capital, fees_owed) =
		pool.burn_limit_order(id.clone(), initick_rup, 0, PoolSide::Asset1).unwrap();

	// DIFF: Burn will have burnt the entire position so it will be deleted.
	assert_eq!(fees_owed, 0);

	// This could be missing + fees_owed[PoolSide::Asset0]
	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(1));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));

	match get_limit_order(&pool, PoolSide::Asset0, initick_rdown, id.clone()) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}
	match get_limit_order(&pool, PoolSide::Asset1, initick_rup, id.clone()) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}
}

// #Burn

fn mediumpool_initialized_zerotick_lo(
) -> (PoolState, PoolAssetMap<AmountU256>, AccountId, Tick, Tick) {
	let id: AccountId = AccountId::from([0xcf; 32]);
	let mut pool = PoolState::new(300, encodedprice1_1()).unwrap();

	let initick_rdown = pool.current_tick;
	let initick_rup = pool.current_tick + TICKSPACING_UNISWAP_MEDIUM;

	let mut minted_capital_accum: PoolAssetMap<AmountU256> = Default::default();

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			initick_rdown,
			INITIALIZE_LIQUIDITY_AMOUNT,
			PoolSide::Asset0,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_capital_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			initick_rup,
			INITIALIZE_LIQUIDITY_AMOUNT,
			PoolSide::Asset1,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_capital_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	(pool, minted_capital_accum, id, initick_rdown, initick_rup)
}

fn checklotickisclear(pool: &PoolState, asset: PoolSide, tick: &Tick) {
	match get_tickinfo_limit_orders(pool, asset, tick) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}
}

fn checkloticknotclear(pool: &PoolState, asset: PoolSide, tick: &Tick) {
	match get_tickinfo_limit_orders(pool, asset, tick) {
		None => panic!("Expected Key"),
		_ => {},
	}
}

// Own test

#[test]
fn multiple_burns_lo() {
	let (mut pool, _, _id, initick_rdown, initick_rup) = mediumpool_initialized_zerotick_lo();
	// some activity that would make the ticks non-zero
	pool.mint_limit_order(
		AccountId::from([0xce; 32]),
		initick_rdown - TICKSPACING_UNISWAP_MEDIUM,
		expandto18decimals(1).as_u128(),
		PoolSide::Asset0,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	pool.mint_limit_order(
		AccountId::from([0xce; 32]),
		initick_rup + TICKSPACING_UNISWAP_MEDIUM,
		expandto18decimals(1).as_u128(),
		PoolSide::Asset1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());
	assert!(pool.swap::<Asset1ToAsset0>(expandto18decimals(1)).is_ok());

	// Should be able to do only 1 burn (1000000000000000000 / 987654321000000000)

	pool.burn_limit_order(
		AccountId::from([0xce; 32]),
		initick_rdown - TICKSPACING_UNISWAP_MEDIUM,
		987654321000000000,
		PoolSide::Asset0,
	)
	.unwrap();

	match pool.burn_limit_order(
		AccountId::from([0xce; 32]),
		initick_rdown - TICKSPACING_UNISWAP_MEDIUM,
		987654321000000000,
		PoolSide::Asset0,
	) {
		Err(PositionError::PositionLacksLiquidity) => {},
		_ => panic!("Expected PositionLacksLiquidity"),
	}

	pool.burn_limit_order(
		AccountId::from([0xce; 32]),
		initick_rup + TICKSPACING_UNISWAP_MEDIUM,
		987654321000000000,
		PoolSide::Asset1,
	)
	.unwrap();

	match pool.burn_limit_order(
		AccountId::from([0xce; 32]),
		initick_rup + TICKSPACING_UNISWAP_MEDIUM,
		987654321000000000,
		PoolSide::Asset1,
	) {
		Err(PositionError::PositionLacksLiquidity) => {},
		_ => panic!("Expected PositionLacksLiquidity"),
	}
}

#[test]
fn notclearposition_ifnomoreliquidity_lo() {
	let (mut pool, _, _id, initick_rdown, initick_rup) = mediumpool_initialized_zerotick_lo();
	let id2 = AccountId::from([0xce; 32]);
	// some activity that would make the ticks non-zero
	pool.mint_limit_order(
		id2.clone(),
		initick_rdown,
		expandto18decimals(1).as_u128(),
		PoolSide::Asset0,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	pool.mint_limit_order(
		id2.clone(),
		initick_rup,
		expandto18decimals(1).as_u128(),
		PoolSide::Asset1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());
	assert!(pool.swap::<Asset1ToAsset0>(expandto18decimals(1)).is_ok());

	// Add a poke to update the fee growth and check it's value
	let (returned_capital, fees_owed) =
		pool.burn_limit_order(id2.clone(), initick_rdown, 0, PoolSide::Asset0).unwrap();

	assert_eq!(fees_owed, 0);
	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));

	let (returned_capital, fees_owed) =
		pool.burn_limit_order(id2.clone(), initick_rup, 0, PoolSide::Asset1).unwrap();

	assert_ne!(fees_owed, 0);
	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));

	let pos0 = get_limit_order(&pool, PoolSide::Asset0, initick_rdown, id2.clone()).unwrap();
	let pos1 = get_limit_order(&pool, PoolSide::Asset0, initick_rup, id2.clone()).unwrap();

	assert_eq!(pos0.liquidity, expandto18decimals(1).as_u128());
	assert_eq!(
		pos0.last_fee_growth_inside,
		U256::from_dec_str("340282366920938463463374607431768211").unwrap()
	);

	assert_eq!(pos1.liquidity, expandto18decimals(1).as_u128());
	assert_eq!(
		pos1.last_fee_growth_inside,
		U256::from_dec_str("340282366920938463463374607431768211").unwrap()
	);

	// Burning partially swapped positions will return swapped position (fees already
	// returned)
	let (returned_capital, fees_owed) = pool
		.burn_limit_order(
			id2.clone(),
			initick_rdown,
			expandto18decimals(1).as_u128(),
			PoolSide::Asset0,
		)
		.unwrap();

	assert_eq!(fees_owed, 0);

	assert_ne!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_ne!(returned_capital[!PoolSide::Asset0], U256::from(0));
	let (returned_capital, fees_owed) = pool
		.burn_limit_order(
			id2.clone(),
			initick_rup,
			expandto18decimals(1).as_u128(),
			PoolSide::Asset1,
		)
		.unwrap();

	assert_eq!(fees_owed, 0);

	assert_ne!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_ne!(returned_capital[!PoolSide::Asset0], U256::from(0));

	match get_limit_order(&pool, PoolSide::Asset0, initick_rdown, id2.clone()) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}

	match get_limit_order(&pool, PoolSide::Asset0, initick_rup, id2.clone()) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}
}

// NOTE: These three tests don't make a lot of sense in limit orders, since there is not
// a low and high tick for each position. So we are just testing that the correct
// positions get burn automatically when swapped

#[test]
fn clearstick_iflastposition_lo() {
	let (mut pool, _, id, _, _) = mediumpool_initialized_zerotick_lo();
	// some activity that would make the ticks non-zero. Different to intiial position in
	// mediumpool_initialized
	let ticklow = pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 5;
	let tickhigh = pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 10;

	// Check that ticks are cleared before minting
	checklotickisclear(&pool, PoolSide::Asset0, &ticklow);
	checklotickisclear(&pool, PoolSide::Asset1, &tickhigh);

	pool.mint_limit_order(id.clone(), ticklow, 1, PoolSide::Asset0, |_| Ok::<(), ()>(()))
		.unwrap();

	pool.mint_limit_order(id.clone(), tickhigh, 1, PoolSide::Asset1, |_| Ok::<(), ()>(()))
		.unwrap();

	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

	pool.burn_limit_order(id.clone(), ticklow, 1, PoolSide::Asset0).unwrap();
	pool.burn_limit_order(id.clone(), ticklow, 1, PoolSide::Asset0).unwrap();

	// Assert position have been burnt
	match get_tickinfo_limit_orders(&pool, PoolSide::Asset0, &ticklow) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}
	match get_tickinfo_limit_orders(&pool, PoolSide::Asset1, &tickhigh) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}

	checklotickisclear(&pool, PoolSide::Asset0, &ticklow);
	checklotickisclear(&pool, PoolSide::Asset1, &tickhigh);
}

// Miscellaneous mint tests

// // Low Fee, tickSpacing = 10, 1:1 price
fn lowpool_initialized_zerotick_lo() -> (PoolState, PoolAssetMap<AmountU256>, AccountId, Tick, Tick)
{
	let id: AccountId = AccountId::from([0xcf; 32]);
	let mut pool = PoolState::new(50, encodedprice1_1()).unwrap();

	let initick_rdown = pool.current_tick;
	let initick_rup = pool.current_tick + TICKSPACING_UNISWAP_LOW;

	let mut minted_capital_accum: PoolAssetMap<AmountU256> = Default::default();

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			initick_rdown,
			INITIALIZE_LIQUIDITY_AMOUNT,
			PoolSide::Asset0,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_capital_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			initick_rup,
			INITIALIZE_LIQUIDITY_AMOUNT,
			PoolSide::Asset1,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	minted_capital_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_capital_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	(pool, minted_capital_accum, id, initick_rdown, initick_rup)
}

#[test]
fn test_mint_rightofcurrentprice_lo() {
	let (mut pool, _, id, _, _) = lowpool_initialized_zerotick_lo();

	let liquiditybefore = pool.current_liquidity.clone();
	let liquidity_delta: u128 = 1000;
	let lowtick: Tick = TICKSPACING_UNISWAP_LOW;
	let uptick: Tick = TICKSPACING_UNISWAP_LOW * 2;

	for tick in lowtick..uptick {
		for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
			let (minted_capital, _) = pool
				.mint_limit_order(id.clone(), tick, liquidity_delta, asset, |_| Ok::<(), ()>(()))
				.unwrap();

			assert_eq!(pool.current_liquidity, liquiditybefore);

			assert_eq!(minted_capital[asset].as_u128(), liquidity_delta);
			assert_eq!(minted_capital[!asset].as_u128(), 0);
		}
	}
}

#[test]
fn test_mint_leftofcurrentprice_lo() {
	let (mut pool, _, id, _, _) = lowpool_initialized_zerotick_lo();

	let liquiditybefore = pool.current_liquidity.clone();
	let liquidity_delta: u128 = 1000;
	let lowtick: Tick = -TICKSPACING_UNISWAP_LOW * 2;
	let uptick: Tick = -TICKSPACING_UNISWAP_LOW;

	for tick in lowtick..uptick {
		for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
			let (minted_capital, _) = pool
				.mint_limit_order(id.clone(), tick, liquidity_delta, asset, |_| Ok::<(), ()>(()))
				.unwrap();

			assert_eq!(pool.current_liquidity, liquiditybefore);

			assert_eq!(minted_capital[asset].as_u128(), liquidity_delta);
			assert_eq!(minted_capital[!asset].as_u128(), 0);
		}
	}
}

#[test]
fn test_mint_withincurrentprice_lo() {
	let (mut pool, _, id, _, _) = lowpool_initialized_zerotick_lo();

	let liquiditybefore = pool.current_liquidity.clone();
	let liquidity_delta: u128 = 1000;
	let lowtick: Tick = -TICKSPACING_UNISWAP_LOW;
	let uptick: Tick = TICKSPACING_UNISWAP_LOW;

	for tick in lowtick..uptick {
		for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
			let (minted_capital, _) = pool
				.mint_limit_order(id.clone(), tick, liquidity_delta, asset, |_| Ok::<(), ()>(()))
				.unwrap();

			assert_eq!(pool.current_liquidity, liquiditybefore);

			assert_eq!(minted_capital[asset].as_u128(), liquidity_delta);
			assert_eq!(minted_capital[!asset].as_u128(), 0);
		}
	}
}

#[test]
fn test_cannotremove_morethanposition_lo() {
	let (mut pool, _, id, initick_rdown, initick_rup) = lowpool_initialized_zerotick_lo();

	match pool.burn_limit_order(
		id.clone(),
		initick_rdown,
		INITIALIZE_LIQUIDITY_AMOUNT + 1,
		PoolSide::Asset0,
	) {
		Err(PositionError::PositionLacksLiquidity) => {},
		_ => panic!("Should not be able to remove more than position"),
	}
	match pool.burn_limit_order(
		id.clone(),
		initick_rup,
		INITIALIZE_LIQUIDITY_AMOUNT + 1,
		PoolSide::Asset1,
	) {
		Err(PositionError::PositionLacksLiquidity) => {},
		_ => panic!("Should not be able to remove more than position"),
	}
}

#[test]
fn test_collectfees_withincurrentprice_lo() {
	let (mut pool, _, id, _, _) = lowpool_initialized_zerotick_lo();

	let liquidity_delta: u128 = 1000;
	let lowtick: Tick = -TICKSPACING_UNISWAP_LOW * 100;
	let uptick: Tick = TICKSPACING_UNISWAP_LOW * 100;

	pool.mint_limit_order(id.clone(), lowtick, liquidity_delta, PoolSide::Asset0, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();
	pool.mint_limit_order(id.clone(), uptick, liquidity_delta, PoolSide::Asset1, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();

	let before_ticklowinfo_lo =
		get_tickinfo_limit_orders(&pool, PoolSide::Asset0, &lowtick).unwrap().clone();

	let before_tickupinfo_lo =
		get_tickinfo_limit_orders(&pool, PoolSide::Asset1, &uptick).unwrap().clone();

	assert!(pool.swap::<Asset0ToAsset1>(expandto18decimals(1)).is_ok());

	// Poke pos0
	let (returned_capital, fees_owed) =
		pool.burn_limit_order(id.clone(), lowtick, 0, PoolSide::Asset0).unwrap();

	let ticklowinfo_lo = get_tickinfo_limit_orders(&pool, PoolSide::Asset0, &lowtick).unwrap();

	assert_eq!(ticklowinfo_lo.liquidity_gross, before_ticklowinfo_lo.liquidity_gross);
	assert_eq!(ticklowinfo_lo.fee_growth_inside, before_ticklowinfo_lo.fee_growth_inside);
	assert_eq!(ticklowinfo_lo.one_minus_percswap, before_ticklowinfo_lo.one_minus_percswap);

	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));
	assert_eq!(fees_owed, 0);

	// Poke pos1
	let (returned_capital, fees_owed) =
		pool.burn_limit_order(id.clone(), lowtick, 0, PoolSide::Asset0).unwrap();

	let tickupinfo_lo = get_tickinfo_limit_orders(&pool, PoolSide::Asset1, &uptick).unwrap();

	assert_eq!(tickupinfo_lo.liquidity_gross, before_tickupinfo_lo.liquidity_gross);
	assert!(tickupinfo_lo.fee_growth_inside > before_tickupinfo_lo.fee_growth_inside);
	assert!(tickupinfo_lo.one_minus_percswap < before_tickupinfo_lo.one_minus_percswap);

	assert_eq!(returned_capital[PoolSide::Asset0], U256::from(0));
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from(0));
	assert!(fees_owed > 0);
}

// Post initialize at medium fee

#[test]
fn test_initial_liquidity_lo() {
	let (pool, _, _, initick_rdown, initick_rup) = mediumpool_initialized_zerotick_lo();

	assert_eq!(
		get_tickinfo_limit_orders(&pool, PoolSide::Asset0, &initick_rdown)
			.unwrap()
			.liquidity_gross +
			get_tickinfo_limit_orders(&pool, PoolSide::Asset1, &initick_rup)
				.unwrap()
				.liquidity_gross,
		INITIALIZE_LIQUIDITY_AMOUNT * 2
	);
}

#[test]
fn test_returns_insupply_inrange_lo() {
	let (mut pool, _, id, _, _) = mediumpool_initialized_zerotick_lo();
	pool.mint_limit_order(
		id.clone(),
		-TICKSPACING_UNISWAP_MEDIUM,
		expandto18decimals(3).as_u128(),
		PoolSide::Asset0,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	pool.mint_limit_order(
		id.clone(),
		TICKSPACING_UNISWAP_MEDIUM,
		expandto18decimals(2).as_u128(),
		PoolSide::Asset1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	assert_eq!(
		get_tickinfo_limit_orders(&pool, PoolSide::Asset0, &-TICKSPACING_UNISWAP_MEDIUM)
			.unwrap()
			.liquidity_gross +
			get_tickinfo_limit_orders(&pool, PoolSide::Asset1, &TICKSPACING_UNISWAP_MEDIUM)
				.unwrap()
				.liquidity_gross,
		expandto18decimals(5).as_u128(),
	);
}

// Uniswap "limit orders"

#[test]
fn test_limitselling_basetopair_tick0thru1_lo() {
	let (mut pool, _, id, _, _) = mediumpool_initialized_zerotick_lo();

	// Value to emulate minted liquidity in Uniswap
	let liquiditytomint: u128 = 5981737760509663;

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			-TICKSPACING_UNISWAP_MEDIUM,
			liquiditytomint,
			PoolSide::Asset0,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset1ToAsset0>((U256::from_dec_str("2000000000000000000").unwrap()).into())
		.is_ok());

	let (burnt, fees_owed) = pool
		.burn_limit_order(
			id.clone(),
			-TICKSPACING_UNISWAP_MEDIUM,
			liquiditytomint,
			PoolSide::Asset0,
		)
		.unwrap();
	assert_eq!(burnt[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	// For now just squaring the sqrt_price_at_tick
	let position_burnt = mul_div_floor(
		U256::from(liquiditytomint),
		PoolState::sqrt_price_at_tick(-TICKSPACING_UNISWAP_MEDIUM)
			.pow(U256::from_dec_str("2").unwrap()),
		U256::from(2).pow(U256::from_dec_str("96").unwrap()),
	);
	assert_eq!(burnt[!PoolSide::Asset0], position_burnt);

	// Original value: 18107525382602. Slightly different because the amount swapped in the
	// position/tick will be slightly different (tick will be crossed with slightly
	// different amounts)
	assert_eq!(fees_owed, 17891544354686);

	match pool.burn_limit_order(id.clone(), -TICKSPACING_UNISWAP_MEDIUM, 0, PoolSide::Asset0) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}
}

#[test]
fn test_limitselling_basetopair_tick0thru1_poke_lo() {
	let (mut pool, _, id, _, _) = mediumpool_initialized_zerotick_lo();

	// Value to emulate minted liquidity in Uniswap
	let liquiditytomint: u128 = 5981737760509663;

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			-TICKSPACING_UNISWAP_MEDIUM,
			liquiditytomint,
			PoolSide::Asset0,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset1ToAsset0>((U256::from_dec_str("2000000000000000000").unwrap()).into())
		.is_ok());

	// Poke
	let (burnt, fees_owed) = pool
		.burn_limit_order(id.clone(), -TICKSPACING_UNISWAP_MEDIUM, 0, PoolSide::Asset0)
		.unwrap();

	assert_eq!(burnt[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burnt[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(fees_owed, 17891544354686);

	let (burnt, fees_owed) = pool
		.burn_limit_order(
			id.clone(),
			-TICKSPACING_UNISWAP_MEDIUM,
			liquiditytomint,
			PoolSide::Asset0,
		)
		.unwrap();
	assert_eq!(burnt[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	// For now just squaring the sqrt_price_at_tick
	let position_burnt = mul_div_floor(
		U256::from(liquiditytomint),
		PoolState::sqrt_price_at_tick(-TICKSPACING_UNISWAP_MEDIUM)
			.pow(U256::from_dec_str("2").unwrap()),
		U256::from(2).pow(U256::from_dec_str("96").unwrap()),
	);
	assert_eq!(burnt[!PoolSide::Asset0], position_burnt);
	assert_eq!(fees_owed, 0);
}

#[test]
fn test_limitselling_pairtobase_tick1thru0_lo() {
	let (mut pool, _, id, _, _) = mediumpool_initialized_zerotick_lo();

	let liquiditytomint: u128 = 5981737760509663;

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			TICKSPACING_UNISWAP_MEDIUM,
			liquiditytomint,
			PoolSide::Asset1,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset0ToAsset1>((U256::from_dec_str("2000000000000000000").unwrap()).into())
		.is_ok());

	let (burnt, fees_owed) = pool
		.burn_limit_order(
			id.clone(),
			TICKSPACING_UNISWAP_MEDIUM,
			expandto18decimals(1).as_u128(),
			PoolSide::Asset1,
		)
		.unwrap();
	assert_eq!(burnt[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	// For now just squaring the sqrt_price_at_tick
	let position_burnt = mul_div_floor(
		U256::from(liquiditytomint),
		U256::from(2).pow(U256::from_dec_str("96").unwrap()),
		PoolState::sqrt_price_at_tick(-TICKSPACING_UNISWAP_MEDIUM)
			.pow(U256::from_dec_str("2").unwrap()),
	);
	assert_eq!(burnt[PoolSide::Asset0], position_burnt);

	// DIFF: position fully burnt
	assert_eq!(fees_owed, 18107525382602);

	match pool.burn_limit_order(id.clone(), TICKSPACING_UNISWAP_MEDIUM, 0, PoolSide::Asset1) {
		Err(PositionError::NonExistent) => {},
		_ => panic!("Expected NonExistent"),
	}
}

#[test]
fn test_limitselling_pairtobase_tick1thru0_poke_lo() {
	let (mut pool, _, id, _, _) = mediumpool_initialized_zerotick_lo();

	let liquiditytomint: u128 = 5981737760509663;

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			TICKSPACING_UNISWAP_MEDIUM,
			liquiditytomint,
			PoolSide::Asset1,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	assert_eq!(minted_capital[!PoolSide::Asset0], U256::from_dec_str("5981737760509663").unwrap());
	assert_eq!(minted_capital[PoolSide::Asset0], U256::from_dec_str("0").unwrap());

	// somebody takes the limit order
	assert!(pool
		.swap::<Asset0ToAsset1>((U256::from_dec_str("2000000000000000000").unwrap()).into())
		.is_ok());

	let (burnt, fees_owed) = pool
		.burn_limit_order(id.clone(), TICKSPACING_UNISWAP_MEDIUM, 0, PoolSide::Asset1)
		.unwrap();

	assert_eq!(burnt[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(burnt[PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	assert_eq!(fees_owed, 18107525382602);

	let (burnt, fees_owed) = pool
		.burn_limit_order(
			id.clone(),
			TICKSPACING_UNISWAP_MEDIUM,
			expandto18decimals(1).as_u128(),
			PoolSide::Asset1,
		)
		.unwrap();
	assert_eq!(burnt[!PoolSide::Asset0], U256::from_dec_str("0").unwrap());
	// For now just squaring the sqrt_price_at_tick
	let position_burnt = mul_div_floor(
		U256::from(liquiditytomint),
		U256::from(2).pow(U256::from_dec_str("96").unwrap()),
		PoolState::sqrt_price_at_tick(-TICKSPACING_UNISWAP_MEDIUM)
			.pow(U256::from_dec_str("2").unwrap()),
	);
	assert_eq!(burnt[PoolSide::Asset0], position_burnt);

	// DIFF: position fully burnt
	assert_eq!(fees_owed, 0);
}

// #Collect

#[test]
fn test_multiplelps_lo() {
	let (mut pool, _, id) = lowpool_initialized_one();
	let id2: AccountId = AccountId::from([0xce; 32]);

	pool.mint_limit_order(
		id.clone(),
		TICKSPACING_UNISWAP_LOW,
		expandto18decimals(1).as_u128(),
		PoolSide::Asset1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	pool.mint_limit_order(
		id2.clone(),
		TICKSPACING_UNISWAP_LOW,
		expandto18decimals(2).as_u128(),
		PoolSide::Asset1,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	assert!(pool.swap::<Asset0ToAsset1>((expandto18decimals(1)).into()).is_ok());

	// poke positions
	let (_, fees_owed) = pool
		.burn_limit_order(id.clone(), TICKSPACING_UNISWAP_LOW, 0, PoolSide::Asset1)
		.unwrap();

	// NOTE: Fee_owed value 1 unit different than Uniswap because uniswap requires 4
	// loops to do the swap instead of 1 causing the rounding to be different
	assert_eq!(fees_owed, 166666666666666 as u128);

	let (_, fees_owed) = pool
		.burn_limit_order(id2.clone(), TICKSPACING_UNISWAP_LOW, 0, PoolSide::Asset1)
		.unwrap();
	// NOTE: Fee_owed value 1 unit different than Uniswap because uniswap requires 4
	// loops to do the swap instead of 1 causing the rounding to be different
	assert_eq!(fees_owed, 333333333333333 as u128);
}

// type(uint128).max * 2**128 / 1e18
// https://www.wolframalpha.com/input/?i=%282**128+-+1%29+*+2**128+%2F+1e18
// U256::from_dec_str("115792089237316195423570985008687907852929702298719625575994"
// ). unwr ap();

// Works across large increases
#[test]
fn test_before_capbidn_lo() {
	let (mut pool, _, id) = lowpool_initialized_one();

	let initick = pool.current_tick;

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), initick, expandto18decimals(1).as_u128(), asset, |_| {
			Ok::<(), ()>(())
		})
		.unwrap();

		let liquidity_map = match asset {
			PoolSide::Asset0 => &mut pool.liquidity_map_base_lo,
			PoolSide::Asset1 => &mut pool.liquidity_map_pair_lo,
		};

		let tickinfo_lo = liquidity_map.get_mut(&initick).unwrap();
		tickinfo_lo.fee_growth_inside =
			U256::from_dec_str("115792089237316195423570985008687907852929702298719625575994")
				.unwrap();

		let (burnt, fees_owed) = pool.burn_limit_order(id.clone(), initick, 0, asset).unwrap();

		assert_eq!(burnt[asset], U256::from_dec_str("0").unwrap());
		assert_eq!(burnt[!asset], U256::from_dec_str("0").unwrap());

		assert_eq!(fees_owed, u128::MAX - 1);
	}
}

#[test]
fn test_after_capbidn_lo() {
	let (mut pool, _, id) = lowpool_initialized_one();

	let initick = pool.current_tick;

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), initick, expandto18decimals(1).as_u128(), asset, |_| {
			Ok::<(), ()>(())
		})
		.unwrap();

		let liquidity_map = match asset {
			PoolSide::Asset0 => &mut pool.liquidity_map_base_lo,
			PoolSide::Asset1 => &mut pool.liquidity_map_pair_lo,
		};

		let tickinfo_lo = liquidity_map.get_mut(&initick).unwrap();
		tickinfo_lo.fee_growth_inside =
			U256::from_dec_str("115792089237316195423570985008687907852929702298719625575995")
				.unwrap();

		let (burnt, fees_owed) = pool.burn_limit_order(id.clone(), initick, 0, asset).unwrap();

		assert_eq!(burnt[asset], U256::from_dec_str("0").unwrap());
		assert_eq!(burnt[!asset], U256::from_dec_str("0").unwrap());

		assert_eq!(fees_owed, u128::MAX);
	}
}

#[test]
fn test_wellafter_capbidn_lo() {
	let (mut pool, _, id) = lowpool_initialized_one();

	let initick = pool.current_tick;

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), initick, expandto18decimals(1).as_u128(), asset, |_| {
			Ok::<(), ()>(())
		})
		.unwrap();

		let liquidity_map = match asset {
			PoolSide::Asset0 => &mut pool.liquidity_map_base_lo,
			PoolSide::Asset1 => &mut pool.liquidity_map_pair_lo,
		};

		let tickinfo_lo = liquidity_map.get_mut(&initick).unwrap();
		tickinfo_lo.fee_growth_inside = U256::MAX;

		let (burnt, fees_owed) = pool.burn_limit_order(id.clone(), initick, 0, asset).unwrap();

		assert_eq!(burnt[asset], U256::from_dec_str("0").unwrap());
		assert_eq!(burnt[!asset], U256::from_dec_str("0").unwrap());

		assert_eq!(fees_owed, u128::MAX);
	}
}

// DIFF: pool.global_fee_growth won't overflow. We make it saturate.

fn lowpool_initialized_setfees_lo() -> (PoolState, PoolAssetMap<AmountU256>, AccountId) {
	let (mut pool, mut minted_amounts_accum, id) = lowpool_initialized_one();
	let id2: AccountId = AccountId::from([0xce; 32]);

	let initick = pool.current_tick;

	// Mint mock positions to initialize tick
	pool.mint_limit_order(id2.clone(), initick, 1, PoolSide::Asset0, |_| Ok::<(), ()>(()))
		.unwrap();
	pool.mint_limit_order(id2.clone(), initick, 1, PoolSide::Asset1, |_| Ok::<(), ()>(()))
		.unwrap();

	// Set fee growth inside to max.
	pool.liquidity_map_base_lo.get_mut(&initick).unwrap().fee_growth_inside = U256::MAX;
	pool.liquidity_map_pair_lo.get_mut(&initick).unwrap().fee_growth_inside = U256::MAX;

	// Initialize positions with fee_growth_inside

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			initick,
			expandto18decimals(10).as_u128(),
			PoolSide::Asset0,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	minted_amounts_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_amounts_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	let (minted_capital, _) = pool
		.mint_limit_order(
			id.clone(),
			initick,
			expandto18decimals(10).as_u128(),
			PoolSide::Asset1,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();

	minted_amounts_accum[PoolSide::Asset0] += minted_capital[PoolSide::Asset0];
	minted_amounts_accum[!PoolSide::Asset0] += minted_capital[!PoolSide::Asset0];

	// Health check
	assert_eq!(minted_amounts_accum[PoolSide::Asset0], expandto18decimals(10));
	assert_eq!(minted_amounts_accum[!PoolSide::Asset0], expandto18decimals(10));

	(pool, minted_amounts_accum, id)
}

#[test]
fn test_base_lo() {
	let (mut pool, _, id) = lowpool_initialized_setfees_lo();

	let initick = pool.current_tick;

	assert!(pool.swap::<Asset1ToAsset0>((expandto18decimals(1)).into()).is_ok());

	let (_, fees_owed) = pool.burn_limit_order(id.clone(), initick, 0, PoolSide::Asset0).unwrap();

	// DIFF: no fees accrued - saturated
	assert_eq!(fees_owed, 0);
}

#[test]
fn test_pair_lo() {
	let (mut pool, _, id) = lowpool_initialized_setfees_lo();

	let initick = pool.current_tick;

	assert!(pool.swap::<Asset0ToAsset1>((expandto18decimals(1)).into()).is_ok());

	let (_, fees_owed) = pool.burn_limit_order(id.clone(), initick, 0, PoolSide::Asset1).unwrap();

	// DIFF: no fees accrued - saturated
	assert_eq!(fees_owed, 0);
}

// Skipped more fee protocol tests

// Medium Fee, tickSpacing = 12, 1:1 price
fn mediumpool_initialized_nomint() -> (PoolState, PoolAssetMap<AmountU256>, AccountId) {
	// fee_pips shall be one order of magnitude smaller than in the Uniswap pool (because
	// ONE_IN_HUNDREDTH_BIPS is /10)
	let pool = PoolState::new(3000, encodedprice1_1()).unwrap();
	let id: AccountId = AccountId::from([0xcf; 32]);
	let minted_amounts: PoolAssetMap<AmountU256> = Default::default();
	(pool, minted_amounts, id)
}
// DIFF: We have a tickspacing of 1, which means we will never have issues with it.
#[test]
fn test_tickspacing_lo() {
	let (mut pool, _, id) = mediumpool_initialized_nomint();

	for asset in vec![PoolSide::Asset0, PoolSide::Asset1] {
		pool.mint_limit_order(id.clone(), -6, 1, asset, |_| Ok::<(), ()>(())).unwrap();
		pool.mint_limit_order(id.clone(), 6, 1, asset, |_| Ok::<(), ()>(())).unwrap();
		pool.mint_limit_order(id.clone(), -12, 1, asset, |_| Ok::<(), ()>(())).unwrap();
		pool.mint_limit_order(id.clone(), 12, 1, asset, |_| Ok::<(), ()>(())).unwrap();
		pool.mint_limit_order(id.clone(), -120, 1, asset, |_| Ok::<(), ()>(())).unwrap();
		pool.mint_limit_order(id.clone(), 120, 1, asset, |_| Ok::<(), ()>(())).unwrap();
		pool.mint_limit_order(id.clone(), -144, 1, asset, |_| Ok::<(), ()>(())).unwrap();
		pool.mint_limit_order(id.clone(), 144, 1, asset, |_| Ok::<(), ()>(())).unwrap();
	}
}

#[test]
fn test_swapping_gaps_pairtobase_lo() {
	let (mut pool, _, id) = mediumpool_initialized_nomint();
	// Change pool current tick so it uses the correct LO orders
	pool.current_tick = 150000;
	let liquidity_amount = 36096898321357 as u128;

	// Mint two orders and check that it uses the correct one.
	// 120192 being the closest tick to the price that is swapped at Uniswap test
	pool.mint_limit_order(id.clone(), 120192, liquidity_amount, PoolSide::Asset0, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();
	pool.mint_limit_order(id.clone(), 121200, liquidity_amount, PoolSide::Asset0, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();

	assert!(pool.swap::<Asset1ToAsset0>((expandto18decimals(1)).into()).is_ok());

	// This order should not have been used

	let (returned_capital, fees_owed) = pool
		.burn_limit_order(id.clone(), 121200, liquidity_amount, PoolSide::Asset0)
		.unwrap();

	assert_eq!(returned_capital[PoolSide::Asset0].as_u128(), liquidity_amount);
	assert_eq!(returned_capital[!PoolSide::Asset0].as_u128(), 0);
	assert_eq!(fees_owed, 0);

	// Poke to get the fees
	let (returned_capital, fees_owed) =
		pool.burn_limit_order(id.clone(), 120192, 0, PoolSide::Asset0).unwrap();

	assert!(fees_owed > 0);

	// Slightly different amounts because of price difference
	// Orig value: 30027458295511
	assert_eq!(returned_capital[PoolSide::Asset0], U256::from_dec_str("30083999478255").unwrap());
	// Substracting fees
	// Orig value: 996999999999848369
	assert_eq!(
		returned_capital[!PoolSide::Asset0],
		U256::from_dec_str("996999999999682559").unwrap()
	);

	// Tick should not have changed
	assert_eq!(pool.current_tick, 150000)
}

#[test]
fn test_swapping_gaps_basetopair_lo() {
	let (mut pool, _, id) = mediumpool_initialized_nomint();
	// Change pool current tick so it uses the correct LO orders
	pool.current_tick = 150000;
	let liquidity_amount = 36096898321357 as u128;

	// Mint two orders and check that it uses the correct one.
	// 120192 being the closest tick to the price that is swapped at Uniswap test
	pool.mint_limit_order(id.clone(), 120192, liquidity_amount, PoolSide::Asset1, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();
	pool.mint_limit_order(id.clone(), 121200, liquidity_amount, PoolSide::Asset1, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();

	assert!(pool.swap::<Asset0ToAsset1>((expandto18decimals(1)).into()).is_ok());

	// This order should not have been used

	let (returned_capital, fees_owed) = pool
		.burn_limit_order(id.clone(), 121200, liquidity_amount, PoolSide::Asset1)
		.unwrap();

	assert_eq!(returned_capital[!PoolSide::Asset0].as_u128(), liquidity_amount);
	assert_eq!(returned_capital[PoolSide::Asset0].as_u128(), 0);
	assert_eq!(fees_owed, 0);

	// Poke to get the fees
	let (returned_capital, fees_owed) =
		pool.burn_limit_order(id.clone(), 120192, 0, PoolSide::Asset1).unwrap();

	assert!(fees_owed > 0);

	// Slightly different amounts because of price difference
	// Orig value: 30027458295511
	assert_eq!(returned_capital[!PoolSide::Asset0], U256::from_dec_str("30083999478255").unwrap());
	// Substracting fees
	// Orig value: 996999999999848369
	assert_eq!(
		returned_capital[PoolSide::Asset0],
		U256::from_dec_str("996999999999682559").unwrap()
	);

	// Tick should not have changed
	assert_eq!(pool.current_tick, 150000)
}

///////////////////////////////////////////////////////////
///             Extra limit order tests                ////
///////////////////////////////////////////////////////////

////// LO Testing utilities //////

// This function will probably be implemented inside the AMM - most likely in a better
// way, as squaring the sqrt price is not optimal.
fn aux_get_price_at_tick(tick: Tick) -> PriceQ128F96 {
	PoolState::sqrt_price_at_tick(tick).pow(U256::from_dec_str("2").unwrap())
}

// Check partially swapped single limit order
// Fully burn a partially swapped position
fn check_swap_one_tick_exactin(
	pool: &mut PoolState,
	amount_swap_in: AmountU256,
	amount_swap_out: AmountU256,
	asset_limit_order: PoolSide,
	price_limit_order: PriceQ128F96,
	total_fee_paid: AmountU256,
) {
	let amount_minus_fees = mul_div_floor(
		amount_swap_in,
		U256::from(ONE_IN_HUNDREDTH_BIPS - pool.fee_100th_bips),
		U256::from(ONE_IN_HUNDREDTH_BIPS),
	); // This cannot overflow as we bound fee_100th_bips to <= ONE_IN_HUNDREDTH_BIPS/2

	assert_eq!(amount_swap_in - amount_minus_fees, total_fee_paid);

	let position_swapped =
		calculate_amount(amount_minus_fees, price_limit_order, asset_limit_order);
	assert_eq!(position_swapped, amount_swap_out);
}

// Fully burn a partially swapped position
fn check_and_burn_limitorder_swap_one_tick_exactin(
	pool: &mut PoolState,
	id: AccountId,
	tick_limit_order: Tick,
	amount_swap_in: AmountU256,
	amount_swap_out: AmountU256,
	asset_limit_order: PoolSide,
	price_limit_order: PriceQ128F96,
	total_fee_paid: AmountU256,
	amount_to_burn: Liquidity,
	tick_to_be_cleared: bool,
) {
	check_swap_one_tick_exactin(
		pool,
		amount_swap_in,
		amount_swap_out,
		asset_limit_order,
		price_limit_order,
		total_fee_paid,
	);

	// Burnt Limit Order
	let (returned_capital, fees_owed) = pool
		.burn_limit_order(id.clone(), tick_limit_order, amount_to_burn, asset_limit_order)
		.unwrap();

	if amount_swap_in == U256::from(0) {
		// No swap happened
		assert_eq!(fees_owed, 0);
	} else {
		// Swap happened
		assert!(fees_owed > 0);
	}

	// This will be the sum of fees and swapped position
	let amount_swapped_in_minus_fees =
		calculate_amount(amount_swap_out, price_limit_order, !asset_limit_order);

	// These checks might be off due to rounding - to check
	assert_eq!(returned_capital[!asset_limit_order], amount_swap_out);
	assert_eq!(U256::from(fees_owed), amount_swap_in - amount_swapped_in_minus_fees);
	assert_eq!(total_fee_paid, U256::from(fees_owed));

	let tick_result = get_tickinfo_limit_orders(pool, asset_limit_order, &tick_limit_order);

	match tick_result {
		Some(_) => assert!(tick_to_be_cleared),
		None => assert!(!tick_to_be_cleared),
	}
}

fn calculate_amount(
	amount_asset_in: AmountU256,
	price_limit_order: PriceQ128F96,
	asset_out: PoolSide,
) -> AmountU256 {
	if asset_out == PoolSide::Asset0 {
		mul_div_floor(amount_asset_in, U256::from(1) << 96u32, price_limit_order)
	} else {
		mul_div_floor(amount_asset_in, price_limit_order, U256::from(1) << 96u32)
	}
}

/////////////////////////////////////////////////////////////////////////////////
///////////////////////// Tests added for limit orders //////////////////////////
/////////////////////////////////////////////////////////////////////////////////

// Initial tick == -23028
// Initially no LO
fn mint_pool_no_lo() -> (PoolState, PoolAssetMap<AmountU256>, AccountId) {
	let mut pool = PoolState::new(300, encodedprice1_10()).unwrap();
	let id: AccountId = AccountId::from([0xcf; 32]);
	let minted_amounts: PoolAssetMap<AmountU256> = Default::default();

	(pool, minted_amounts, id)
}
// Skipped collect tests
#[test]
fn test_swap_asset0_to_asset1_partial_swap_lo() {
	let (mut pool, _, id) = mint_pool_no_lo();
	partial_swap_lo(&mut pool, id, PoolSide::Asset0);
}

#[test]
fn test_swap_asset1_to_asset0_partial_swap_lo() {
	let (mut pool, _, id) = mint_pool_no_lo();
	partial_swap_lo(&mut pool, id, PoolSide::Asset1);
}

fn partial_swap_lo(
	pool: &mut PoolState,
	id: AccountId,
	asset_in: PoolSide,
) -> (Tick, U256, U256, U256, u128) {
	let ini_liquidity = pool.current_liquidity;
	let ini_tick = pool.current_tick;
	let ini_price = pool.current_sqrt_price;

	let tick_limit_order = if asset_in == PoolSide::Asset1 {
		pool.current_tick - TICKSPACING_UNISWAP_MEDIUM * 10
	} else {
		pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 10
	};

	let liquidity_amount = expandto18decimals(1).as_u128();

	// Limit order should partially been swapped
	let price_limit_order = aux_get_price_at_tick(tick_limit_order);
	// Pool has been initialized at around 1 : 10
	let price_ini = aux_get_price_at_tick(ini_tick);

	if asset_in == PoolSide::Asset0 {
		// Check that lo price is > than initial price
		assert!(price_limit_order > price_ini);
	} else {
		// Check that lo price is < than initial price
		assert!(price_limit_order < price_ini);
	}

	pool.mint_limit_order(id.clone(), tick_limit_order, liquidity_amount, !asset_in, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();

	let amount_to_swap = expandto18decimals(1) / 10;

	let mut total_amount_out = Default::default();
	let mut total_fee_paid = Default::default();
	if asset_in == PoolSide::Asset0 {
		(total_amount_out, total_fee_paid) =
			pool.swap::<Asset0ToAsset1>((amount_to_swap).into()).unwrap();
	} else {
		(total_amount_out, total_fee_paid) =
			pool.swap::<Asset1ToAsset0>((amount_to_swap).into()).unwrap();
	}

	// Check swap outcomes
	// Tick, sqrtPrice and liquidity haven't changed (range order pool)
	assert_eq!(pool.current_liquidity, ini_liquidity);
	assert_eq!(pool.current_tick, ini_tick);
	assert_eq!(pool.current_sqrt_price, ini_price);

	check_swap_one_tick_exactin(
		pool,
		amount_to_swap,
		total_amount_out,
		!asset_in,
		price_limit_order,
		total_fee_paid,
	);
	(tick_limit_order, amount_to_swap, total_amount_out, total_fee_paid, liquidity_amount)
}

#[test]
fn test_swap_asset0_to_asset1_full_swap_lo() {
	let (mut pool, _, id) = mint_pool_no_lo();
	full_swap_lo(&mut pool, id, PoolSide::Asset0);
}

#[test]
fn test_swap_asset1_to_asset0_full_swap_lo() {
	let (mut pool, _, id) = mint_pool_no_lo();
	full_swap_lo(&mut pool, id, PoolSide::Asset1);
}

fn full_swap_lo(
	pool: &mut PoolState,
	id: AccountId,
	asset_in: PoolSide,
) -> (Tick, Tick, U256, U256) {
	let id2: AccountId = AccountId::from([0xce; 32]);

	let ini_liquidity = pool.current_liquidity;
	let ini_tick = pool.current_tick;
	let ini_price = pool.current_sqrt_price;

	let (tick_limit_order_0, tick_limit_order_1) = if asset_in == PoolSide::Asset1 {
		(
			pool.current_tick - TICKSPACING_UNISWAP_MEDIUM * 10,
			pool.current_tick - TICKSPACING_UNISWAP_MEDIUM * 2,
		)
	} else {
		(
			pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 10,
			pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 2,
		)
	};

	let liquidity_amount = expandto18decimals(1).as_u128();

	pool.mint_limit_order(id2.clone(), tick_limit_order_0, liquidity_amount, !asset_in, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();

	pool.mint_limit_order(id.clone(), tick_limit_order_1, liquidity_amount, !asset_in, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();

	let amount_to_swap = expandto18decimals(1) / 15;
	let mut total_amount_out = Default::default();
	let mut total_fee_paid = Default::default();
	if asset_in == PoolSide::Asset0 {
		(total_amount_out, total_fee_paid) =
			pool.swap::<Asset0ToAsset1>((amount_to_swap).into()).unwrap();
	} else {
		(total_amount_out, total_fee_paid) =
			pool.swap::<Asset1ToAsset0>((amount_to_swap).into()).unwrap();
	}

	// This should have partially swapped the limit order placed
	let price_limit_order_0 = aux_get_price_at_tick(tick_limit_order_0);
	let price_limit_order_1 = aux_get_price_at_tick(tick_limit_order_1);
	// Pool has been initialized at around 1 : 10
	let price_ini = aux_get_price_at_tick(ini_tick);

	if asset_in == PoolSide::Asset0 {
		// Check that lo price is > than initial price
		assert!(price_limit_order_0 > price_ini);
		assert!(price_limit_order_0 > price_limit_order_1);
	} else {
		// Check that lo price is < than initial price
		assert!(price_limit_order_0 < price_ini);
		assert!(price_limit_order_0 < price_limit_order_1);
	}

	// Check swap outcomes
	// Tick, sqrtPrice and liquidity haven't changed (range order pool)
	assert_eq!(pool.current_liquidity, ini_liquidity);
	assert_eq!(pool.current_tick, ini_tick);
	assert_eq!(pool.current_sqrt_price, ini_price);

	let amount_minus_fees = mul_div_floor(
		amount_to_swap,
		U256::from(ONE_IN_HUNDREDTH_BIPS - pool.fee_100th_bips),
		U256::from(ONE_IN_HUNDREDTH_BIPS),
	);

	// Part will be swapped from tickLO and part from tickLO1. Price will be worse than if
	// it was fully swapped from tickLO but better than if it was fully swapped in tick LO1
	let amount_out_iff_limit_order_0 =
		calculate_amount(amount_minus_fees, price_limit_order_0, !asset_in);
	let amount_out_iff_limit_order_1 =
		calculate_amount(amount_minus_fees, price_limit_order_0, !asset_in);

	assert!(total_amount_out < amount_out_iff_limit_order_0);
	assert!(total_amount_out > amount_out_iff_limit_order_1);

	// Check LO position and tick
	match get_limit_order(&pool, !asset_in, tick_limit_order_0, id2.clone()) {
		None => {},
		_ => panic!("Expected NonExistent Key"),
	}

	let tick_1 = get_tickinfo_limit_orders(&pool, !asset_in, &tick_limit_order_1).unwrap();
	let liquidity_left = tick_1.liquidity_gross * (tick_1.one_minus_percswap).as_u128();

	assert_eq!(U256::from(liquidity_left), U256::from(liquidity_amount) * 2 - total_amount_out);
	(tick_limit_order_0, tick_limit_order_1, total_amount_out, total_fee_paid)
}

#[test]
fn test_mint_worse_lo_asset0_for_asset1() {
	mint_worse_lo_swap(PoolSide::Asset0);
}

#[test]
fn test_mint_worse_lo_asset1_for_asset0() {
	mint_worse_lo_swap(PoolSide::Asset1);
}

fn mint_worse_lo_swap(asset_in: PoolSide) {
	let (mut pool, _, id) = mint_pool_no_lo();

	let tick_to_mint =
		if asset_in == PoolSide::Asset1 { pool.current_tick - 1 } else { pool.current_tick + 1 };

	pool.mint_limit_order(
		id.clone(),
		tick_to_mint,
		expandto18decimals(1).as_u128(),
		!asset_in,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	partial_swap_lo(&mut pool, id.clone(), asset_in);

	// Check LO position and tick
	match get_limit_order(&pool, !asset_in, tick_to_mint, id.clone()) {
		None => panic!("Expected existant Key"),
		Some(limit_order) => {
			// Limit order shouldn't have been used
			assert_eq!(limit_order.liquidity, expandto18decimals(1).as_u128());
			assert_eq!(limit_order.last_one_minus_percswap, U256::from(1));
		},
	}
	match get_tickinfo_limit_orders(&pool, !asset_in, &tick_to_mint) {
		None => panic!("Expected existant Key"),
		Some(tick) => {
			// Tick should have been used
			assert_eq!(tick.liquidity_gross, expandto18decimals(1).as_u128());
			assert_eq!(tick.one_minus_percswap, U256::from(1));
		},
	}
}

#[test]
fn test_multiple_positions_asset0_for_asset1() {
	multiple_positions(PoolSide::Asset0);
}

#[test]
fn test_multiple_positions_asset1_for_asset0() {
	multiple_positions(PoolSide::Asset1);
}

fn multiple_positions(asset_in: PoolSide) {
	let (mut pool, _, id) = mint_pool_no_lo();
	let id2: AccountId = AccountId::from([0xce; 32]);

	let tick_to_mint = if asset_in == PoolSide::Asset1 {
		pool.current_tick - TICKSPACING_UNISWAP_MEDIUM * 10
	} else {
		pool.current_tick + TICKSPACING_UNISWAP_MEDIUM * 10
	};

	let initial_liquidity = expandto18decimals(1).as_u128();

	pool.mint_limit_order(id.clone(), tick_to_mint, initial_liquidity, !asset_in, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();
	pool.mint_limit_order(id2.clone(), tick_to_mint, initial_liquidity, !asset_in, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();

	// Check tick before swapping
	match get_tickinfo_limit_orders(&pool, !asset_in, &tick_to_mint) {
		None => panic!("Expected existant Key"),
		Some(tick) => {
			assert_eq!(tick.liquidity_gross, initial_liquidity * 2);
			assert_eq!(tick.one_minus_percswap, U256::from(1));
		},
	}

	let amount_to_swap = expandto18decimals(10);

	// To cross the first tick (=== first position tickL0) and part of the second (tickL01)
	let (total_amount_out, total_fee_paid) = if asset_in == PoolSide::Asset0 {
		pool.swap::<Asset0ToAsset1>(amount_to_swap).unwrap()
	} else {
		pool.swap::<Asset1ToAsset0>(amount_to_swap).unwrap()
	};

	check_swap_one_tick_exactin(
		&mut pool,
		amount_to_swap,
		total_amount_out,
		asset_in,
		aux_get_price_at_tick(tick_to_mint),
		total_fee_paid,
	);

	// Check position and tick
	match get_limit_order(&pool, !asset_in, tick_to_mint, id.clone()) {
		None => panic!("Expected existant Key"),
		Some(limit_order) => {
			assert_eq!(limit_order.liquidity, expandto18decimals(1).as_u128());
			assert_eq!(limit_order.last_one_minus_percswap, U256::from(1));
		},
	}
	match get_limit_order(&pool, !asset_in, tick_to_mint, id2.clone()) {
		None => panic!("Expected existant Key"),
		Some(limit_order) => {
			assert_eq!(limit_order.liquidity, expandto18decimals(1).as_u128());
			assert_eq!(limit_order.last_one_minus_percswap, U256::from(1));
		},
	}

	match get_tickinfo_limit_orders(&pool, !asset_in, &tick_to_mint) {
		None => panic!("Expected existant Key"),
		Some(tick) => {
			assert_eq!(tick.liquidity_gross, initial_liquidity);
			assert!(tick.one_minus_percswap < U256::from(1));
		},
	}
}

// Skipped tests for ownerPositions - unclear how we will do that.
// from test_chainflipPool.py line 2869 to 2955

#[test]
fn test_mint_partially_swapped_tick_asset0_for_asset1() {
	let (mut pool, _, id) = mint_pool_no_lo();
	mint_partially_swapped_tick(&mut pool, id, PoolSide::Asset0);
}

#[test]
fn test_mint_partially_swapped_tick_asset1_for_asset0() {
	let (mut pool, _, id) = mint_pool_no_lo();
	mint_partially_swapped_tick(&mut pool, id, PoolSide::Asset1);
}

fn mint_partially_swapped_tick(
	pool: &mut PoolState,
	id: AccountId,
	asset_in: PoolSide,
) -> (Tick, U256, U256, U256, u128) {
	let id2 = AccountId::from([0xce; 32]);
	let (tick_to_mint, amount_swap_in, amount_swap_out, total_fee_paid, liquidity_amount) =
		partial_swap_lo(pool, id.clone(), asset_in);

	let tick_info = get_tickinfo_limit_orders(&pool, !asset_in, &tick_to_mint).unwrap();

	let ini_liquidity_gross = tick_info.liquidity_gross;
	let ini_one_minus_perc_swapped = tick_info.one_minus_percswap;
	assert_eq!(ini_liquidity_gross, expandto18decimals(1).as_u128());
	assert!(ini_one_minus_perc_swapped < expandto18decimals(1));

	pool.mint_limit_order(
		id2.clone(),
		tick_to_mint,
		expandto18decimals(1).as_u128(),
		!asset_in,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();
	let tick_info = get_tickinfo_limit_orders(&pool, !asset_in, &tick_to_mint).unwrap();
	assert_eq!(tick_info.liquidity_gross, expandto18decimals(1).as_u128());
	assert_eq!(tick_info.one_minus_percswap, ini_one_minus_perc_swapped);
	assert_eq!(
		get_limit_order(&pool, !asset_in, tick_to_mint, id2.clone()).unwrap().liquidity,
		expandto18decimals(1).as_u128()
	);
	(tick_to_mint, amount_swap_in, amount_swap_out, total_fee_paid, liquidity_amount)
}

#[test]
fn test_mint_fully_swapped_tick_diff_account_asset0_for_asset1() {
	mint_fully_swapped_tick_diff_account(PoolSide::Asset0);
}

#[test]
fn test_mint_fully_swapped_tick_diff_account_asset1_for_asset0() {
	mint_fully_swapped_tick_diff_account(PoolSide::Asset1);
}

fn mint_fully_swapped_tick_diff_account(asset_in: PoolSide) {
	let (mut pool, _, id) = mint_pool_no_lo();
	let id3: AccountId = AccountId::from([0xcc; 32]);

	let (tick_limit_order_0, tick_limit_order_1, total_amount_out_0, total_fee_paid_0) =
		full_swap_lo(&mut pool, id.clone(), asset_in);

	// Check that tick_limit_order_1 is partially swapped and not removed
	let tick_info = get_tickinfo_limit_orders(&pool, !asset_in, &tick_limit_order_1).unwrap();
	assert!(tick_info.liquidity_gross > 0);
	assert!(tick_info.one_minus_percswap < U256::from(1));

	// Mint a position on top of the previous fully swapped position
	pool.mint_limit_order(
		id3.clone(),
		tick_limit_order_0,
		expandto18decimals(1).as_u128(),
		!asset_in,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	let amount_to_swap = expandto18decimals(10);

	// Fully swap the newly minted position and part of the backup position
	// (tick_limit_order_1)
	let (total_amount_out_1, total_fee_paid_1) = if asset_in == PoolSide::Asset0 {
		pool.swap::<Asset0ToAsset1>(amount_to_swap).unwrap()
	} else {
		pool.swap::<Asset1ToAsset0>(amount_to_swap).unwrap()
	};

	// Check that the results are the same as in the first swap
	assert_eq!(total_amount_out_0, total_amount_out_1);
	assert_eq!(total_fee_paid_0, total_fee_paid_1);
}

#[test]
fn test_burn_position_minted_after_swap_asset0_for_asset1() {
	burn_position_minted_after_swap(PoolSide::Asset0);
}

#[test]
fn test_burn_position_minted_after_swap_asset1_for_asset0() {
	burn_position_minted_after_swap(PoolSide::Asset1);
}

fn burn_position_minted_after_swap(asset_in: PoolSide) {
	let (mut pool, _, id) = mint_pool_no_lo();
	let id2 = AccountId::from([0xcc; 32]);
	let (tick_to_mint, amount_swap_in, amount_swap_out, total_fee_paid, liquidity_position) =
		mint_partially_swapped_tick(&mut pool, id.clone(), asset_in);

	// Burn newly minted position
	check_and_burn_limitorder_swap_one_tick_exactin(
		&mut pool,
		id2.clone(),
		tick_to_mint,
		amount_swap_in,
		amount_swap_out,
		!asset_in,
		aux_get_price_at_tick(tick_to_mint),
		total_fee_paid,
		0,
		false,
	);

	// Check amounts and first position (partially swapped) - same check as in
	// test_swap0For1_partialSwap for the first minted position. Nothing should have changed
	// by minting and burning an extra position on top after the swap has taken place.
	check_and_burn_limitorder_swap_one_tick_exactin(
		&mut pool,
		id.clone(),
		tick_to_mint,
		amount_swap_in,
		amount_swap_out,
		!asset_in,
		aux_get_price_at_tick(tick_to_mint),
		U256::from(0), // fee collected in the poke
		liquidity_position,
		true,
	);
}

#[test]
fn test_limitorder_currenttick() {
	let (mut pool, _, id) = mediumpool_initialized_nomint();

	let ini_tick = pool.current_tick;
	// Check no limit order exists
	assert!(pool.liquidity_map_base_lo.is_empty());
	assert!(pool.liquidity_map_pair_lo.is_empty());

	// Loop through the two assets, minting a position and check the tick info
	for asset in [PoolSide::Asset0, PoolSide::Asset1].iter() {
		pool.mint_limit_order(
			id.clone(),
			ini_tick,
			expandto18decimals(1).as_u128(),
			*asset,
			|_| Ok::<(), ()>(()),
		)
		.unwrap();
	}
	let tick_info_0 = get_tickinfo_limit_orders(&pool, PoolSide::Asset0, &ini_tick).unwrap();
	let tick_info_1 = get_tickinfo_limit_orders(&pool, PoolSide::Asset1, &ini_tick).unwrap();

	assert_eq!(tick_info_0.liquidity_gross, expandto18decimals(1).as_u128());
	assert_eq!(tick_info_0.one_minus_percswap, U256::from(1));
	assert_eq!(tick_info_1.liquidity_gross, expandto18decimals(1).as_u128());
	assert_eq!(tick_info_1.one_minus_percswap, U256::from(1));

	// Swap asset0 for asset1
	assert!(pool.swap::<Asset1ToAsset0>((expandto18decimals(1)).into()).is_ok());

	assert_eq!(pool.current_tick, ini_tick);

	let tick_info_0 = get_tickinfo_limit_orders(&pool, PoolSide::Asset0, &ini_tick).unwrap();
	let tick_info_1 = get_tickinfo_limit_orders(&pool, PoolSide::Asset1, &ini_tick).unwrap();

	// Tick 1 not altered
	assert_eq!(tick_info_1.liquidity_gross, expandto18decimals(1).as_u128());
	assert_eq!(tick_info_1.one_minus_percswap, U256::from(1));

	// In one direction the limit order is taken
	assert_eq!(tick_info_0.liquidity_gross, expandto18decimals(1).as_u128());
	// Should be almost zero (not zero bc there are fees). Just checking that it has been
	// used.
	assert!(tick_info_0.one_minus_percswap < U256::from(1));

	// Swap asset1 for asset0
	assert!(pool.swap::<Asset1ToAsset0>((expandto18decimals(1)).into()).is_ok());

	let tick_info_0 = get_tickinfo_limit_orders(&pool, PoolSide::Asset0, &ini_tick).unwrap();
	let tick_info_1 = get_tickinfo_limit_orders(&pool, PoolSide::Asset1, &ini_tick).unwrap();

	// In the other direction it is taken but not until the range orders don't change the
	// pool price
	assert_ne!(pool.current_tick, ini_tick);
	// Not ending at the border (MIN_TICK) but rather going to the next best LO tick - 1
	assert_eq!(pool.current_tick, ini_tick - 1);

	// Tick 0 not altered
	assert_eq!(tick_info_0.liquidity_gross, expandto18decimals(1).as_u128());
	assert!(tick_info_0.one_minus_percswap < U256::from(1));

	// Tick1 used
	assert_eq!(tick_info_1.liquidity_gross, expandto18decimals(1).as_u128());
	// Should be almost zero (not zero bc there are fees). Just checking that it has been
	// used.
	assert!(tick_info_1.one_minus_percswap < U256::from(1));
}

#[test]
fn test_no_rangeorder_limitorder_worseprice_asset0() {
	no_rangeorder_limitorder_worseprice(PoolSide::Asset0);
}
#[test]
fn test_no_rangeorder_limitorder_worseprice_asset1() {
	no_rangeorder_limitorder_worseprice(PoolSide::Asset1);
}

fn no_rangeorder_limitorder_worseprice(asset_in: PoolSide) {
	let (mut pool, _, id) = mediumpool_initialized_nomint();

	// Tick == 0
	let ini_tick = pool.current_tick;

	let tick_limit_order = if asset_in == PoolSide::Asset0 {
		ini_tick - TICKSPACING_UNISWAP_MEDIUM * 10
	} else {
		ini_tick + TICKSPACING_UNISWAP_MEDIUM * 10
	};

	pool.mint_limit_order(
		id.clone(),
		tick_limit_order,
		expandto18decimals(1).as_u128(),
		!asset_in,
		|_| Ok::<(), ()>(()),
	)
	.unwrap();

	assert_ne!(pool.current_tick, ini_tick);

	// Order should be taken but not until the range orders don't change the pool price.
	// Not ending at the border but rather going to the next best LO tick.
	if asset_in == PoolSide::Asset1 {
		assert_eq!(pool.current_tick, tick_limit_order);
	} else {
		assert_eq!(pool.current_tick, tick_limit_order - 1);
	}
	assert!(
		get_tickinfo_limit_orders(&pool, !asset_in, &ini_tick)
			.unwrap()
			.one_minus_percswap <
			U256::from(1)
	);
	assert_eq!(
		get_tickinfo_limit_orders(&pool, asset_in, &ini_tick)
			.unwrap()
			.one_minus_percswap,
		U256::from(1)
	);
}

#[test]
fn test_burn_partiallyswapped_multiplesteps_asset0() {
	burn_partiallyswapped_multiplesteps(PoolSide::Asset0);
}

#[test]
fn test_burn_partiallyswapped_multiplesteps_asset1() {
	burn_partiallyswapped_multiplesteps(PoolSide::Asset1);
}

fn burn_partiallyswapped_multiplesteps(asset_in: PoolSide) {
	let (mut pool, _, id) = mint_pool_no_lo();
	let (tick_minted, amount_swap_in, amount_swap_out, total_fee_paid, liquidity_amount) =
		partial_swap_lo(&mut pool, id.clone(), asset_in);

	let mut pool_copy = pool.clone();

	let (returned_capital_0, fees_owed_0) = pool_copy
		.burn_limit_order(id.clone(), tick_minted, expandto18decimals(1).as_u128(), !asset_in)
		.unwrap();

	match pool_copy.burn_limit_order(id.clone(), tick_minted, 1, !asset_in) {
		Ok(_) => panic!("Should not be able to burn more than minted"),
		Err(PositionError::NonExistent) => {},
		Err(_) => panic!("Wrong error"),
	}

	// Arbitrary numbers (2,4)
	for i in 2..=4 {
		let mut pool_copy = pool.clone();
		let mut returned_capital_1_accum = returned_capital_0.clone(); // just to initialize
		let mut fees_owed_1_accum: u128 = Default::default();
		// Loop for value of i
		for j in 0..i {
			// Fees owed will be returned in the first iteration
			let (returned_capital_1, fees_owed_1) = pool_copy
				.burn_limit_order(
					id.clone(),
					tick_minted,
					expandto18decimals(1).as_u128() / i,
					!asset_in,
				)
				.unwrap();
			returned_capital_1_accum[PoolSide::Asset0] += returned_capital_1[PoolSide::Asset0];
			returned_capital_1_accum[PoolSide::Asset1] += returned_capital_1[PoolSide::Asset1];
			fees_owed_1_accum += fees_owed_1;
		}
		match pool_copy.burn_limit_order(id.clone(), tick_minted, 1, !asset_in) {
			Ok(_) => panic!("Should not be able to burn more than minted"),
			Err(PositionError::NonExistent) => {},
			Err(_) => panic!("Wrong error"),
		}
		// There can be a small rounding error in favour of the pool when burning in
		// multiple steps
		assert_eq!(
			returned_capital_0[PoolSide::Asset0],
			returned_capital_1_accum[PoolSide::Asset0] + 1
		);
		assert_eq!(
			returned_capital_0[PoolSide::Asset1],
			returned_capital_1_accum[PoolSide::Asset1] + 1
		);
		assert_eq!(fees_owed_0, fees_owed_1_accum);
	}
}

#[test]
fn test_mint_on_swapped_position_asset0() {
	mint_on_swapped_position(PoolSide::Asset0);
}
#[test]
fn test_mint_on_swapped_position_asset1() {
	mint_on_swapped_position(PoolSide::Asset1);
}

fn mint_on_swapped_position(asset_in: PoolSide) {
	let (mut pool, _, id) = mint_pool_no_lo();
	let (tick_minted, amount_swap_in, amount_swap_out, total_fee_paid, liquidity_amount) =
		partial_swap_lo(&mut pool, id.clone(), asset_in);

	let mut pool_copy = pool.clone();

	// Amount of swapped tokens that should get burnt regardless of newly
	// minted orders on top
	let (returned_capital_0, fees_owed_0) = pool_copy
		.burn_limit_order(id.clone(), tick_minted, liquidity_amount, !asset_in)
		.unwrap();

	pool.mint_limit_order(id.clone(), tick_minted, liquidity_amount * 1000, !asset_in, |_| {
		Ok::<(), ()>(())
	})
	.unwrap();

	assert_eq!(
		get_limit_order(&pool, !asset_in, tick_minted, id.clone()).unwrap().liquidity,
		liquidity_amount * (1000 + 1)
	);
	// Burn to check if now the entire position gets swapped by the percentatge
	// swapped in the first swap
	let (returned_capital_1, fees_owed_1) = pool
		.burn_limit_order(id.clone(), tick_minted, liquidity_amount * (1001), !asset_in)
		.unwrap();

	assert_eq!(returned_capital_0[asset_in], returned_capital_1[asset_in]);
	assert_eq!(
		returned_capital_0[!asset_in],
		returned_capital_1[!asset_in] - liquidity_amount * 1000
	);
	assert_eq!(fees_owed_0, fees_owed_1);
}
