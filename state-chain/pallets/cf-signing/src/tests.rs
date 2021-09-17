use crate::{self as pallet_cf_signing, mock::*, Error};
use frame_support::traits::Hooks;
use frame_support::{assert_ok, assert_noop};
use frame_support::instances::Instance0;
use frame_system::pallet_prelude::BlockNumberFor;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Scenario {
	HappyPath,
	RetryPath,
}

pub const SIGNATURE: &'static str = "Wow!";

struct MockCfe;

impl MockCfe {
	fn respond(scenario: Scenario) {
		for event_record in System::events() {
			Self::process_event(event_record.event, scenario);
		}
	}

	fn process_event(event: Event, scenario: Scenario) {
		match event {
			Event::pallet_cf_signing_Instance0(
				pallet_cf_signing::Event::ThresholdSignatureRequest(
					req_id,
					key_id,
					signers,
					payload,
				),
			) => {
				assert_eq!(key_id, DOGE_KEY_ID);
				assert_eq!(signers, vec![RANDOM_NOMINEE]);
				assert_eq!(payload, DOGE_PAYLOAD);

				let result = match scenario {
					Scenario::HappyPath => DogeSigning::signature_success(Origin::root(), req_id, SIGNATURE.to_string()),
					Scenario::RetryPath => DogeSigning::signature_failed(Origin::root(), req_id, vec![RANDOM_NOMINEE]),
				};
				assert_ok!(result);
			}
			_ => panic!("Unexpected event"),
		};
	}
}

#[test]
fn happy_path() {
	new_test_ext().execute_with(|| {
		// Initiate request
		let request_id = DogeSigning::request_signature(DogeSigningContext {
			message: "Amazing!".to_string(),
		});
		let pending = DogeSigning::pending_request(request_id).unwrap();
		assert_eq!(pending.attempt, 0);
		assert_eq!(pending.signatories, vec![RANDOM_NOMINEE]);

		// Wrong request id is a no-op
		assert_noop!(
			DogeSigning::signature_success(
				Origin::root(),
				request_id + 1,
				"MaliciousSignature".to_string()
			),
			Error::<Test, Instance0>::InvalidRequestId
		);

		// CFE responds
		MockCfe::respond(Scenario::HappyPath);

		// Request is complete
		assert!(DogeSigning::pending_request(request_id).is_none());
		
		// Call back has executed.
		assert_eq!(
			MockCallback::<DogeSigningContext>::get_stored_callback(),
			Some("So Amazing! Such Wow!".to_string())
		);
	});
}

#[test]
fn retry_path() {
	new_test_ext().execute_with(|| {
		// Initiate request
		let request_id = DogeSigning::request_signature(DogeSigningContext {
			message: "Amazing!".to_string(),
		});
		let pending = DogeSigning::pending_request(request_id).unwrap();
		assert_eq!(pending.attempt, 0);
		assert_eq!(pending.signatories, vec![RANDOM_NOMINEE]);

		// CFE responds
		MockCfe::respond(Scenario::RetryPath);

		// Request is complete
		assert!(DogeSigning::pending_request(request_id).is_none());

		// Call back has *not* executed.
		assert_eq!(MockCallback::<DogeSigningContext>::get_stored_callback(), None);

		// The offender has been reported.
		assert_eq!(MockOfflineConditions::get_reported(), vec![RANDOM_NOMINEE]);

		// Scheduled for retry.
		assert_eq!(DogeSigning::retry_queue().len(), 1);
		
		// Process retries.
		<DogeSigning as Hooks<BlockNumberFor<Test>>>::on_initialize(0);

		// No longer pending retry.
		assert!(DogeSigning::retry_queue().is_empty());

		// We have a new request pending.
		let pending = DogeSigning::pending_request(request_id + 1).unwrap();
		assert_eq!(pending.attempt, 1);
		assert_eq!(pending.signatories, vec![RANDOM_NOMINEE]);
	});
}
