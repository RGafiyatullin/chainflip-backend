#![cfg(test)]

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok, traits::HandleLifetime};
use frame_system::Provider;

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;

#[test]
fn test_ensure_origin_struct() {
	new_test_ext().execute_with(|| {
		SwappingEnabled::<Test>::put(true);
		// Root and none should be invalid.
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::root()).unwrap_err();
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::none()).unwrap_err();
		EnsureValidator::<Test>::ensure_origin(OriginFor::<Test>::root()).unwrap_err();
		EnsureValidator::<Test>::ensure_origin(OriginFor::<Test>::none()).unwrap_err();
		EnsureLiquidityProvider::<Test>::ensure_origin(OriginFor::<Test>::root()).unwrap_err();
		EnsureLiquidityProvider::<Test>::ensure_origin(OriginFor::<Test>::none()).unwrap_err();

		// Validation should fail for non-existent accounts.
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::signed(ALICE)).unwrap_err();
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::signed(BOB)).unwrap_err();
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::signed(CHARLIE)).unwrap_err();

		// Create the accounts.
		<Provider<Test> as HandleLifetime<u64>>::created(&ALICE).unwrap();
		<Provider<Test> as HandleLifetime<u64>>::created(&BOB).unwrap();
		<Provider<Test> as HandleLifetime<u64>>::created(&CHARLIE).unwrap();

		// Validation should fail for uninitalised accounts.
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::signed(ALICE)).unwrap_err();
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::signed(BOB)).unwrap_err();
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::signed(CHARLIE)).unwrap_err();

		// Upgrade the accounts.
		Pallet::<Test>::register_as_broker(&ALICE).unwrap();
		Pallet::<Test>::register_as_validator(&BOB).unwrap();
		Pallet::<Test>::register_as_liquidity_provider(&CHARLIE).unwrap();

		// Each account should validate as the correct account type and fail otherwise.
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::signed(ALICE)).unwrap();
		EnsureValidator::<Test>::ensure_origin(OriginFor::<Test>::signed(ALICE)).unwrap_err();
		EnsureLiquidityProvider::<Test>::ensure_origin(OriginFor::<Test>::signed(ALICE))
			.unwrap_err();
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::signed(BOB)).unwrap_err();
		EnsureValidator::<Test>::ensure_origin(OriginFor::<Test>::signed(BOB)).unwrap();
		EnsureLiquidityProvider::<Test>::ensure_origin(OriginFor::<Test>::signed(BOB)).unwrap_err();
		EnsureBroker::<Test>::ensure_origin(OriginFor::<Test>::signed(CHARLIE)).unwrap_err();
		EnsureValidator::<Test>::ensure_origin(OriginFor::<Test>::signed(CHARLIE)).unwrap_err();
		EnsureLiquidityProvider::<Test>::ensure_origin(OriginFor::<Test>::signed(CHARLIE)).unwrap();
	});
}

#[test]
fn test_ensure_origin_fn() {
	new_test_ext().execute_with(|| {
		SwappingEnabled::<Test>::put(true);
		// Root and none should be invalid.
		ensure_broker::<Test>(OriginFor::<Test>::root()).unwrap_err();
		ensure_broker::<Test>(OriginFor::<Test>::none()).unwrap_err();
		ensure_validator::<Test>(OriginFor::<Test>::root()).unwrap_err();
		ensure_validator::<Test>(OriginFor::<Test>::none()).unwrap_err();
		ensure_liquidity_provider::<Test>(OriginFor::<Test>::root()).unwrap_err();
		ensure_liquidity_provider::<Test>(OriginFor::<Test>::none()).unwrap_err();

		// Validation should fail for non-existent accounts.
		ensure_broker::<Test>(OriginFor::<Test>::signed(ALICE)).unwrap_err();
		ensure_broker::<Test>(OriginFor::<Test>::signed(BOB)).unwrap_err();
		ensure_broker::<Test>(OriginFor::<Test>::signed(CHARLIE)).unwrap_err();

		// Create the accounts.
		<Provider<Test> as HandleLifetime<u64>>::created(&ALICE).unwrap();
		<Provider<Test> as HandleLifetime<u64>>::created(&BOB).unwrap();
		<Provider<Test> as HandleLifetime<u64>>::created(&CHARLIE).unwrap();

		// Validation should fail for uninitalised accounts.
		ensure_broker::<Test>(OriginFor::<Test>::signed(ALICE)).unwrap_err();
		ensure_broker::<Test>(OriginFor::<Test>::signed(BOB)).unwrap_err();
		ensure_broker::<Test>(OriginFor::<Test>::signed(CHARLIE)).unwrap_err();

		// Upgrade the accounts.
		Pallet::<Test>::register_as_broker(&ALICE).unwrap();
		Pallet::<Test>::register_as_validator(&BOB).unwrap();
		Pallet::<Test>::register_as_liquidity_provider(&CHARLIE).unwrap();

		// Each account should validate as the correct account type and fail otherwise.
		<Pallet<Test> as AccountRoleRegistry<Test>>::ensure_broker(OriginFor::<Test>::signed(
			ALICE,
		))
		.unwrap();
		<Pallet<Test> as AccountRoleRegistry<Test>>::ensure_validator(OriginFor::<Test>::signed(
			ALICE,
		))
		.unwrap_err();
		<Pallet<Test> as AccountRoleRegistry<Test>>::ensure_liquidity_provider(
			OriginFor::<Test>::signed(ALICE),
		)
		.unwrap_err();
		<Pallet<Test> as AccountRoleRegistry<Test>>::ensure_broker(OriginFor::<Test>::signed(BOB))
			.unwrap_err();
		<Pallet<Test> as AccountRoleRegistry<Test>>::ensure_validator(OriginFor::<Test>::signed(
			BOB,
		))
		.unwrap();
		<Pallet<Test> as AccountRoleRegistry<Test>>::ensure_liquidity_provider(
			OriginFor::<Test>::signed(BOB),
		)
		.unwrap_err();
		<Pallet<Test> as AccountRoleRegistry<Test>>::ensure_broker(OriginFor::<Test>::signed(
			CHARLIE,
		))
		.unwrap_err();
		<Pallet<Test> as AccountRoleRegistry<Test>>::ensure_validator(OriginFor::<Test>::signed(
			CHARLIE,
		))
		.unwrap_err();
		<Pallet<Test> as AccountRoleRegistry<Test>>::ensure_liquidity_provider(
			OriginFor::<Test>::signed(CHARLIE),
		)
		.unwrap();
	});
}

#[test]
fn cannot_register_swapping_roles_if_swapping_disabled() {
	new_test_ext().execute_with(|| {
		assert!(!SwappingEnabled::<Test>::get());

		// As if the account is already funded.
		AccountRoles::<Test>::insert(ALICE, AccountRole::Unregistered);

		assert_noop!(Pallet::<Test>::register_as_broker(&ALICE), Error::<Test>::SwappingDisabled);
		assert_noop!(
			Pallet::<Test>::register_as_liquidity_provider(&ALICE),
			Error::<Test>::SwappingDisabled
		);

		// We can still register as a validator.
		assert_ok!(Pallet::<Test>::register_as_validator(&ALICE));
	});
}
