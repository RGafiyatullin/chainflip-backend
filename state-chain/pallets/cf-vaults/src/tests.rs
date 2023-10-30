#![cfg(test)]

use crate::{mock::*, PendingVaultActivation, VaultActivationStatus};
use cf_chains::mocks::MockAggKey;
use cf_test_utilities::last_event;
use cf_traits::{AsyncResult, SafeMode, VaultActivator};
use frame_support::assert_ok;
use sp_core::Get;

pub const NEW_AGG_PUBKEY: MockAggKey = MockAggKey(*b"newk");

macro_rules! assert_last_event {
	($pat:pat) => {
		let event = last_event::<Test>();
		assert!(
			matches!(event, $crate::mock::RuntimeEvent::VaultsPallet($pat)),
			"Unexpected event {:?}",
			event
		);
	};
}

#[test]
fn test_vault_key_rotated_externally_triggers_code_red() {
	new_test_ext().execute_with(|| {
		const TX_HASH: [u8; 4] = [0xab; 4];
		assert_eq!(<MockRuntimeSafeMode as Get<MockRuntimeSafeMode>>::get(), SafeMode::CODE_GREEN);
		assert_ok!(VaultsPallet::vault_key_rotated_externally(
			RuntimeOrigin::root(),
			NEW_AGG_PUBKEY,
			1,
			TX_HASH,
		));
		assert_eq!(<MockRuntimeSafeMode as Get<MockRuntimeSafeMode>>::get(), SafeMode::CODE_RED);
		assert_last_event!(crate::Event::VaultRotatedExternally(..));
	});
}

#[test]
fn key_unavailable_on_activate_returns_governance_event() {
	new_test_ext_no_key().execute_with(|| {
		VaultsPallet::activate(NEW_AGG_PUBKEY, None, false);

		assert_last_event!(crate::Event::AwaitingGovernanceActivation { .. });

		// we're awaiting the governance action, so we are pending from
		// perspective of an outside observer (e.g. the validator pallet)
		assert_eq!(VaultsPallet::status(), AsyncResult::Pending);
	});
}

#[test]
fn when_set_agg_key_with_agg_key_not_required_we_skip_to_completion() {
	new_test_ext().execute_with(|| {
		MockSetAggKeyWithAggKey::set_required(false);

		VaultsPallet::activate(NEW_AGG_PUBKEY, Some(Default::default()), false);

		assert!(matches!(
			PendingVaultActivation::<Test, _>::get().unwrap(),
			VaultActivationStatus::Complete
		))
	});
}

// add test for testing active from block functionality
