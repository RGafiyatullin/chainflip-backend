#![cfg(test)]
use cf_chains::btc::{api::UtxoSelectionType, deposit_address::DepositAddress, ScriptPubkey, Utxo};
use cf_primitives::SemVer;
use cf_traits::SafeMode;
use frame_support::{assert_ok, traits::OriginTrait};

use crate::{RuntimeSafeMode, SafeModeUpdate};

use crate::mock::*;

#[test]
fn genesis_config() {
	new_test_ext().execute_with(|| {
		assert_eq!(STATE_CHAIN_GATEWAY_ADDRESS, Environment::state_chain_gateway_address());
		assert_eq!(ETH_KEY_MANAGER_ADDRESS, Environment::key_manager_address());
		assert_eq!(ARB_KEY_MANAGER_ADDRESS, Environment::arb_key_manager_address());
		assert_eq!(ETH_CHAIN_ID, Environment::ethereum_chain_id());
		assert_eq!(ARB_CHAIN_ID, Environment::arbitrum_chain_id());
	});
}

#[test]
fn test_btc_utxo_selection() {
	const SCRIPT_PUBKEY: ScriptPubkey = ScriptPubkey::Taproot([0u8; 32]);

	let utxo = |amount| Utxo {
		amount,
		id: Default::default(),
		deposit_address: DepositAddress::new(Default::default(), Default::default()),
	};

	new_test_ext().execute_with(|| {
		// returns none when there are no utxos available for selection
		assert_eq!(
			Environment::select_and_take_bitcoin_utxos(UtxoSelectionType::SelectAllForRotation),
			None
		);

		// add some UTXOs to the available utxos list.
		Environment::add_bitcoin_utxo_to_list(10000, Default::default(), SCRIPT_PUBKEY);
		Environment::add_bitcoin_utxo_to_list(5000, Default::default(), SCRIPT_PUBKEY);
		Environment::add_bitcoin_utxo_to_list(100000, Default::default(), SCRIPT_PUBKEY);
		Environment::add_bitcoin_utxo_to_list(5000000, Default::default(), SCRIPT_PUBKEY);
		Environment::add_bitcoin_utxo_to_list(25000, Default::default(), SCRIPT_PUBKEY);

		// select some utxos for a tx
		assert_eq!(
			Environment::select_and_take_bitcoin_utxos(UtxoSelectionType::Some {
				output_amount: 12000,
				number_of_outputs: 2
			})
			.unwrap(),
			(vec![utxo(5000), utxo(10000), utxo(25000), utxo(100000)], 120080)
		);

		// add the change utxo back to the available utxo list
		Environment::add_bitcoin_utxo_to_list(120080, Default::default(), SCRIPT_PUBKEY);

		// select all remaining utxos
		assert_eq!(
			Environment::select_and_take_bitcoin_utxos(UtxoSelectionType::SelectAllForRotation)
				.unwrap(),
			(vec![utxo(5000000), utxo(120080),], 5116060)
		);

		// add some more utxos to the list
		Environment::add_bitcoin_utxo_to_list(5000, Default::default(), SCRIPT_PUBKEY);
		Environment::add_bitcoin_utxo_to_list(15000, Default::default(), SCRIPT_PUBKEY);

		// request a larger amount than what is available
		assert!(Environment::select_and_take_bitcoin_utxos(UtxoSelectionType::Some {
			output_amount: 20100,
			number_of_outputs: 1
		})
		.is_none());

		// Ensure the previous failure didn't wipe the utxo list
		assert_eq!(
			Environment::select_and_take_bitcoin_utxos(UtxoSelectionType::SelectAllForRotation)
				.unwrap(),
			(vec![utxo(5000), utxo(15000),], 15980)
		);
	});
}

#[test]
fn update_safe_mode() {
	new_test_ext().execute_with(|| {
		// Default to GREEN
		assert_eq!(RuntimeSafeMode::<Test>::get(), SafeMode::CODE_GREEN);
		assert_ok!(Environment::update_safe_mode(OriginTrait::root(), SafeModeUpdate::CodeRed));
		assert_eq!(RuntimeSafeMode::<Test>::get(), SafeMode::CODE_RED);
		System::assert_last_event(RuntimeEvent::Environment(
			crate::Event::<Test>::RuntimeSafeModeUpdated { safe_mode: SafeModeUpdate::CodeRed },
		));

		assert_ok!(Environment::update_safe_mode(OriginTrait::root(), SafeModeUpdate::CodeGreen,));
		assert_eq!(RuntimeSafeMode::<Test>::get(), SafeMode::CODE_GREEN);
		System::assert_last_event(RuntimeEvent::Environment(
			crate::Event::<Test>::RuntimeSafeModeUpdated { safe_mode: SafeModeUpdate::CodeGreen },
		));
		let mock_code_amber =
			MockRuntimeSafeMode { mock: MockPalletSafeMode { flag1: true, flag2: false } };
		assert_ok!(Environment::update_safe_mode(
			OriginTrait::root(),
			SafeModeUpdate::CodeAmber(mock_code_amber.clone())
		));
		assert_eq!(RuntimeSafeMode::<Test>::get(), mock_code_amber);
		System::assert_last_event(RuntimeEvent::Environment(
			crate::Event::<Test>::RuntimeSafeModeUpdated {
				safe_mode: SafeModeUpdate::CodeAmber(mock_code_amber),
			},
		));
	});
}

#[test]
fn can_set_next_compatibility_version() {
	new_test_ext().execute_with(|| {
		assert!(Environment::next_compatibility_version().is_none());

		// Set the next cfe version
		let version = Some(SemVer { major: 1u8, minor: 3u8, patch: 10u8 });
		assert_ok!(Environment::set_next_compatibility_version(RuntimeOrigin::root(), version));
		assert_eq!(Environment::next_compatibility_version(), version);
		System::assert_last_event(RuntimeEvent::Environment(
			crate::Event::<Test>::NextCompatibilityVersionSet { version },
		));

		// Unset the net cfe version
		assert_ok!(Environment::set_next_compatibility_version(RuntimeOrigin::root(), None));
		assert!(Environment::next_compatibility_version().is_none());
		System::assert_last_event(RuntimeEvent::Environment(
			crate::Event::<Test>::NextCompatibilityVersionSet { version: None },
		));
	});
}
