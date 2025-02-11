#![cfg(feature = "runtime-benchmarks")]

use super::*;

use cf_traits::{AccountRoleRegistry, Chainflip};
use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok,
	traits::{EnsureOrigin, OnNewAccount, UnfilteredDispatchable},
};
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn funded() {
		let amount: T::Amount = T::Amount::from(100u32);
		let withdrawal_address: EthereumAddress = Default::default();
		let tx_hash: pallet::EthTransactionHash = [211u8; 32];
		let caller: T::AccountId = whitelisted_caller();

		let call = Call::<T>::funded {
			account_id: caller.clone(),
			amount,
			funder: withdrawal_address,
			tx_hash,
		};
		let origin = T::EnsureWitnessed::try_successful_origin().unwrap();

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(origin));
		}

		assert_eq!(T::Flip::balance(&caller), amount);
	}

	#[benchmark]
	fn redeem() {
		// If we redeem an amount which takes us below the minimum balance, the redemption
		// will fail.
		let balance_to_redeem = RedemptionAmount::Exact(T::Amount::from(2u32));
		let tx_hash: pallet::EthTransactionHash = [211u8; 32];
		let withdrawal_address: EthereumAddress = Default::default();

		let caller: T::AccountId = whitelisted_caller();
		let origin = T::EnsureWitnessed::try_successful_origin().unwrap();

		let call = Call::<T>::funded {
			account_id: caller.clone(),
			amount: MinimumFunding::<T>::get() * T::Amount::from(2u32),
			funder: withdrawal_address,
			tx_hash,
		};
		assert_ok!(call.dispatch_bypass_filter(origin));

		#[extrinsic_call]
		redeem(
			RawOrigin::Signed(caller.clone()),
			balance_to_redeem,
			withdrawal_address,
			Default::default(),
		);

		assert!(PendingRedemptions::<T>::contains_key(&caller));
	}

	#[benchmark]
	fn redeem_all() {
		let withdrawal_address: EthereumAddress = Default::default();
		let tx_hash: pallet::EthTransactionHash = [211u8; 32];
		let caller: T::AccountId = whitelisted_caller();
		let origin = T::EnsureWitnessed::try_successful_origin().unwrap();

		assert_ok!(Call::<T>::funded {
			account_id: caller.clone(),
			amount: MinimumFunding::<T>::get(),
			funder: withdrawal_address,
			tx_hash
		}
		.dispatch_bypass_filter(origin));

		let call = Call::<T>::redeem {
			amount: RedemptionAmount::Max,
			address: withdrawal_address,
			executor: Default::default(),
		};

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into()));
		}

		assert!(PendingRedemptions::<T>::contains_key(&caller));
	}

	#[benchmark]
	fn redeemed() {
		let tx_hash: pallet::EthTransactionHash = [211u8; 32];
		let withdrawal_address: EthereumAddress = Default::default();

		let caller: T::AccountId = whitelisted_caller();
		let origin = T::EnsureWitnessed::try_successful_origin().unwrap();
		let funds = MinimumFunding::<T>::get() * T::Amount::from(10u32);

		assert_ok!(Call::<T>::funded {
			account_id: caller.clone(),
			amount: funds,
			funder: withdrawal_address,
			tx_hash
		}
		.dispatch_bypass_filter(origin.clone()));

		// Push a redemption
		let redeemed_amount = funds / T::Amount::from(2u32);
		assert_ok!(Pallet::<T>::redeem(
			RawOrigin::Signed(caller.clone()).into(),
			RedemptionAmount::Exact(redeemed_amount),
			withdrawal_address,
			Default::default()
		));

		let call = Call::<T>::redeemed { account_id: caller.clone(), redeemed_amount, tx_hash };

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(origin));
		}

		assert!(!PendingRedemptions::<T>::contains_key(&caller));
	}

	#[benchmark]
	fn redemption_expired() {
		let tx_hash: pallet::EthTransactionHash = [211u8; 32];
		let withdrawal_address: EthereumAddress = Default::default();

		let caller: T::AccountId = whitelisted_caller();
		let origin = T::EnsureWitnessed::try_successful_origin().unwrap();

		assert_ok!(Call::<T>::funded {
			account_id: caller.clone(),
			amount: MinimumFunding::<T>::get(),
			funder: withdrawal_address,
			tx_hash
		}
		.dispatch_bypass_filter(origin.clone()));

		// Push a redemption
		assert_ok!(Pallet::<T>::redeem(
			RawOrigin::Signed(caller.clone()).into(),
			RedemptionAmount::Max,
			withdrawal_address,
			Default::default()
		));

		let call =
			Call::<T>::redemption_expired { account_id: caller.clone(), block_number: 2u32.into() };

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(origin));
		}

		assert!(!PendingRedemptions::<T>::contains_key(&caller));
	}

	#[benchmark]
	fn stop_bidding() {
		let caller: T::AccountId = whitelisted_caller();
		<T as frame_system::Config>::OnNewAccount::on_new_account(&caller);
		T::AccountRoleRegistry::register_as_validator(&caller).unwrap();
		ActiveBidder::<T>::insert(caller.clone(), true);

		#[extrinsic_call]
		stop_bidding(RawOrigin::Signed(caller.clone()));

		assert!(!ActiveBidder::<T>::get(caller));
	}

	#[benchmark]
	fn start_bidding() {
		let caller: T::AccountId = whitelisted_caller();
		<T as frame_system::Config>::OnNewAccount::on_new_account(&caller);
		T::AccountRoleRegistry::register_as_validator(&caller).unwrap();
		ActiveBidder::<T>::insert(caller.clone(), false);

		#[extrinsic_call]
		start_bidding(RawOrigin::Signed(caller.clone()));

		assert!(ActiveBidder::<T>::get(caller));
	}

	#[benchmark]
	fn update_minimum_funding() {
		let call =
			Call::<T>::update_minimum_funding { minimum_funding: MinimumFunding::<T>::get() };

		let origin = <T as Chainflip>::EnsureGovernance::try_successful_origin().unwrap();

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(origin));
		}

		assert_eq!(MinimumFunding::<T>::get(), MinimumFunding::<T>::get());
	}

	#[benchmark]
	fn update_redemption_tax() {
		let amount = 1u32.into();
		let call = Call::<T>::update_redemption_tax { amount };

		#[block]
		{
			assert_ok!(
				call.dispatch_bypass_filter(T::EnsureGovernance::try_successful_origin().unwrap())
			);
		}

		assert_eq!(crate::RedemptionTax::<T>::get(), amount);
	}

	#[benchmark]
	fn bind_redeem_address() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		bind_redeem_address(RawOrigin::Signed(caller.clone()), Default::default());

		assert!(BoundRedeemAddress::<T>::contains_key(&caller));
	}

	#[benchmark]
	fn update_restricted_addresses(a: Linear<1, 100>, b: Linear<1, 100>, c: Linear<1, 100>) {
		for i in 0..c {
			let some_balance = FlipBalance::<T>::from(100_u32);
			let some_account: AccountId<T> = account("doogle", 0, i);
			let balances: BTreeMap<EthereumAddress, FlipBalance<T>> =
				BTreeMap::from([(Default::default(), some_balance)]);
			RestrictedBalances::<T>::insert(some_account, balances);
		}
		let call = Call::<T>::update_restricted_addresses {
			addresses_to_add: (1..a as u32).map(|_| Default::default()).collect::<Vec<_>>(),
			addresses_to_remove: (1..b as u32).map(|_| Default::default()).collect::<Vec<_>>(),
		};

		#[block]
		{
			assert_ok!(
				call.dispatch_bypass_filter(T::EnsureGovernance::try_successful_origin().unwrap())
			);
		}
	}

	#[benchmark]
	fn bind_executor_address() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		bind_executor_address(RawOrigin::Signed(caller.clone()), Default::default());

		assert!(BoundExecutorAddress::<T>::contains_key(&caller));
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
}
