#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::dispatch::UnfilteredDispatchable;

benchmarks! {
	enable_swapping {
		let origin = <T as Config>::EnsureGovernance::successful_origin();
		let call = Call::<T>::enable_swapping{};
	}: {
		call.dispatch_bypass_filter(origin)?;
	}
	verify {
		assert!(SwappingEnabled::<T>::get());
	}
	gov_register_account_role {
		let origin = <T as Config>::EnsureGovernance::successful_origin();
		let caller: T::AccountId = whitelisted_caller();
		Pallet::<T>::on_new_account(&caller);

		let call = Call::<T>::gov_register_account_role{ account: caller.clone(), role: AccountRole::Broker };
	}: {
		call.dispatch_bypass_filter(origin)?;
	}
	verify {
		assert_eq!(AccountRoles::<T>::get(&caller), Some(AccountRole::Broker));
	}
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
}
