use crate::{mock::*, EarnedRelayerFees, Pallet, Swap, SwapQueue, WeightInfo};
use cf_chains::AnyChain;
use cf_primitives::{Asset, ForeignChain, ForeignChainAddress};
use cf_test_utilities::assert_event_sequence;
use cf_traits::{mocks::egress_handler::MockEgressHandler, SwapIntentHandler};
use frame_support::{assert_ok, sp_std::iter};

use frame_support::traits::Hooks;

// Returns some test data
fn generate_test_swaps() -> Vec<Swap> {
	vec![
		Swap {
			swap_id: 1,
			from: Asset::Flip,
			to: Asset::Usdc,
			amount: 100,
			egress_address: ForeignChainAddress::Eth([2; 20]),
		},
		Swap {
			swap_id: 2,
			from: Asset::Flip,
			to: Asset::Usdc,
			amount: 200,
			egress_address: ForeignChainAddress::Eth([4; 20]),
		},
		Swap {
			swap_id: 3,
			from: Asset::Flip,
			to: Asset::Usdc,
			amount: 300,
			egress_address: ForeignChainAddress::Eth([7; 20]),
		},
		Swap {
			swap_id: 4,
			from: Asset::Eth,
			to: Asset::Usdc,
			amount: 40,
			egress_address: ForeignChainAddress::Eth([9; 20]),
		},
		Swap {
			swap_id: 5,
			from: Asset::Flip,
			to: Asset::Eth,
			amount: 500,
			egress_address: ForeignChainAddress::Eth([2; 20]),
		},
		Swap {
			swap_id: 6,
			from: Asset::Flip,
			to: Asset::Dot,
			amount: 600,
			egress_address: ForeignChainAddress::Dot([4; 32]),
		},
	]
}

fn insert_swaps(swaps: &[Swap]) {
	for (relayer_id, swap) in swaps.iter().enumerate() {
		assert_ok!(<Pallet<Test> as SwapIntentHandler>::schedule_swap(
			ForeignChainAddress::Eth([2; 20]),
			swap.from,
			swap.to,
			swap.amount,
			swap.egress_address,
			relayer_id as u64,
			2,
		));
	}
}

#[test]
fn register_swap_intent_success_with_valid_parameters() {
	new_test_ext().execute_with(|| {
		assert_ok!(Swapping::register_swap_intent(
			Origin::signed(ALICE),
			Asset::Eth,
			Asset::Usdc,
			ForeignChainAddress::Eth(Default::default()),
			0,
		));
	});
}

#[test]
fn process_all_swaps() {
	new_test_ext().execute_with(|| {
		let swaps = generate_test_swaps();
		insert_swaps(&swaps);
		Swapping::on_idle(
			1,
			<() as WeightInfo>::execute_group_of_swaps(swaps.len() as u32) * (swaps.len() as u64),
		);
		assert!(SwapQueue::<Test>::get().is_empty());
		let mut expected = swaps
			.iter()
			.cloned()
			.map(|swap| (swap.to, swap.amount, swap.egress_address))
			.collect::<Vec<_>>();
		expected.sort();
		let mut egresses = MockEgressHandler::<AnyChain>::get_scheduled_egresses();
		egresses.sort();
		for (input, output) in iter::zip(expected, egresses) {
			assert_eq!(input, output);
		}
	});
}

#[test]
fn number_of_swaps_processed_limited_by_weight() {
	new_test_ext().execute_with(|| {
		let swaps = generate_test_swaps();
		insert_swaps(&swaps);
		Swapping::on_idle(1, 0);
		assert_eq!(SwapQueue::<Test>::get().len(), swaps.len());
	});
}

#[test]
fn expect_earned_fees_to_be_recorded() {
	new_test_ext().execute_with(|| {
		const ALICE: u64 = 2_u64;
		const BOB: u64 = 3_u64;
		assert_ok!(<Pallet<Test> as SwapIntentHandler>::schedule_swap(
			ForeignChainAddress::Eth([2; 20]),
			Asset::Flip,
			Asset::Usdc,
			100,
			ForeignChainAddress::Eth([2; 20]),
			ALICE,
			200,
		));
		assert_ok!(<Pallet<Test> as SwapIntentHandler>::schedule_swap(
			ForeignChainAddress::Eth([2; 20]),
			Asset::Flip,
			Asset::Usdc,
			500,
			ForeignChainAddress::Eth([2; 20]),
			BOB,
			100,
		));
		Swapping::on_idle(1, 1000);
		assert_eq!(EarnedRelayerFees::<Test>::get(ALICE, cf_primitives::Asset::Flip), 2);
		assert_eq!(EarnedRelayerFees::<Test>::get(BOB, cf_primitives::Asset::Flip), 5);
		assert_ok!(<Pallet<Test> as SwapIntentHandler>::schedule_swap(
			ForeignChainAddress::Eth([2; 20]),
			Asset::Flip,
			Asset::Usdc,
			100,
			ForeignChainAddress::Eth([2; 20]),
			ALICE,
			200,
		));
		Swapping::on_idle(1, 1000);
		assert_eq!(EarnedRelayerFees::<Test>::get(ALICE, cf_primitives::Asset::Flip), 4);
	});
}

#[test]
#[should_panic]
fn cannot_swap_with_incorrect_egress_address_type() {
	new_test_ext().execute_with(|| {
		const ALICE: u64 = 1_u64;
		let _ = <Pallet<Test> as SwapIntentHandler>::schedule_swap(
			ForeignChainAddress::Eth([2; 20]),
			Asset::Eth,
			Asset::Dot,
			10,
			ForeignChainAddress::Eth([2; 20]),
			ALICE,
			2,
		);
	});
}

#[test]
fn expect_swap_id_to_be_emitted() {
	new_test_ext().execute_with(|| {
		// 1. Register a swap intent -> NewSwapIntent
		assert_ok!(Swapping::register_swap_intent(
			Origin::signed(ALICE),
			Asset::Eth,
			Asset::Usdc,
			ForeignChainAddress::Eth(Default::default()),
			0,
		));
		// 2. Schedule the swap -> SwapIngressReceived
		assert_ok!(<Pallet<Test> as SwapIntentHandler>::schedule_swap(
			ForeignChainAddress::Eth(Default::default()),
			Asset::Flip,
			Asset::Usdc,
			500,
			ForeignChainAddress::Eth(Default::default()),
			ALICE,
			0,
		));
		// 3. Process swaps -> SwapExecuted, SwapEgressScheduled
		Swapping::on_idle(1, 100);
		assert_event_sequence!(
			Test,
			crate::mock::Event::Swapping(crate::Event::NewSwapIntent {
				ingress_address: ForeignChainAddress::Eth(Default::default()),
			}),
			crate::mock::Event::Swapping(crate::Event::SwapIngressReceived {
				ingress_address: ForeignChainAddress::Eth(Default::default()),
				swap_id: 1,
				ingress_amount: 500
			}),
			crate::mock::Event::Swapping(crate::Event::SwapExecuted { swap_id: 1 }),
			crate::mock::Event::Swapping(crate::Event::SwapEgressScheduled {
				swap_id: 1,
				egress_id: (ForeignChain::Ethereum, 1),
				egress_amount: 500
			})
		);
	});
}
