use std::collections::BTreeSet;

use rand::SeedableRng;

use crate::{
	bitcoin,
	client::{
		common::{
			BroadcastFailureReason, DelayDeserialization, SigningFailureReason, SigningStageName,
		},
		helpers::{
			gen_dummy_local_sig, gen_dummy_signing_comm1, new_nodes, new_signing_ceremony,
			run_stages, PayloadAndKeyData, SigningCeremonyRunner, ACCOUNT_IDS,
			DEFAULT_SIGNING_CEREMONY_ID,
		},
		keygen::generate_key_data,
		signing::signing_data,
	},
	crypto::{bitcoin::BtcSigning, polkadot::PolkadotSigning},
	eth::EthSigning,
	CryptoScheme, Rng,
};

// We choose (arbitrarily) to use eth crypto for unit tests.
use crate::crypto::eth::Point;
type VerifyComm2 = signing_data::VerifyComm2<Point>;
type LocalSig3 = signing_data::LocalSig3<Point>;
type VerifyLocalSig4 = signing_data::VerifyLocalSig4<Point>;

mod broadcast_commitments_stage {

	use super::*;

	#[tokio::test]
	async fn should_report_on_inconsistent_broadcast() {
		let (mut signing_ceremony, _) = new_signing_ceremony::<EthSigning>().await;

		let mut messages = signing_ceremony.request().await;

		// This account id will "broadcast" inconsistently
		let [bad_account_id] = signing_ceremony.select_account_ids();
		for message in messages.get_mut(&bad_account_id).unwrap().values_mut() {
			*message = gen_dummy_signing_comm1(&mut signing_ceremony.rng, 1);
		}

		let messages = signing_ceremony.run_stage::<VerifyComm2, _, _>(messages).await;
		signing_ceremony.distribute_messages(messages).await;
		signing_ceremony
			.complete_with_error(
				&[bad_account_id],
				SigningFailureReason::BroadcastFailure(
					BroadcastFailureReason::Inconsistency,
					SigningStageName::VerifyCommitmentsBroadcast2,
				),
			)
			.await;
	}

	#[tokio::test]
	async fn should_report_on_deserialization_failure() {
		use crate::client::common::DelayDeserialization;

		let (mut signing_ceremony, _) = new_signing_ceremony::<EthSigning>().await;

		let mut messages = signing_ceremony.request().await;

		let [bad_account_id] = signing_ceremony.select_account_ids();
		for message in messages.get_mut(&bad_account_id).unwrap().values_mut() {
			*message = DelayDeserialization::new(&"Not a valid Comm1");
		}

		let messages = signing_ceremony.run_stage::<VerifyComm2, _, _>(messages).await;
		signing_ceremony.distribute_messages(messages).await;
		signing_ceremony
			.complete_with_error(&[bad_account_id], SigningFailureReason::DeserializationError)
			.await;
	}

	#[tokio::test]
	async fn should_report_on_invalid_number_of_commitments() {
		// A party that send the wrong number of commitments (not matching
		// the number of payloads) should be reported

		let (mut signing_ceremony, _) = new_signing_ceremony::<BtcSigning>().await;

		let mut messages = signing_ceremony.request().await;

		let [bad_account_id] = signing_ceremony.select_account_ids();
		for message in messages.get_mut(&bad_account_id).unwrap().values_mut() {
			*message = {
				// Deserialize, add an (unexpected) extra commitment and serialize again:
				let mut comm1 = message.clone().deserialize().unwrap();
				comm1.0.push(comm1.0[0].clone());
				DelayDeserialization::new(&comm1)
			}
		}

		let messages = signing_ceremony.run_stage::<VerifyComm2, _, _>(messages).await;
		signing_ceremony.distribute_messages(messages).await;
		signing_ceremony
			.complete_with_error(&[bad_account_id], SigningFailureReason::InvalidNumberOfPayloads)
			.await;
	}
}

mod local_signatures_stage {

	use super::*;

	#[tokio::test]
	async fn should_report_on_inconsistent_broadcast() {
		let (mut signing_ceremony, _) = new_signing_ceremony::<EthSigning>().await;

		let messages = signing_ceremony.request().await;

		let mut messages = run_stages!(signing_ceremony, messages, VerifyComm2, LocalSig3);

		// This account id will send an invalid signature
		let [bad_account_id] = signing_ceremony.select_account_ids();
		for message in messages.get_mut(&bad_account_id).unwrap().values_mut() {
			*message = gen_dummy_local_sig(&mut signing_ceremony.rng);
		}

		let messages = signing_ceremony.run_stage::<VerifyLocalSig4, _, _>(messages).await;
		signing_ceremony.distribute_messages(messages).await;
		signing_ceremony
			.complete_with_error(
				&[bad_account_id],
				SigningFailureReason::BroadcastFailure(
					BroadcastFailureReason::Inconsistency,
					SigningStageName::VerifyLocalSigsBroadcastStage4,
				),
			)
			.await;
	}

	#[tokio::test]
	async fn should_report_on_invalid_local_signature() {
		let (mut signing_ceremony, _) = new_signing_ceremony::<EthSigning>().await;

		let messages = signing_ceremony.request().await;
		let mut messages = run_stages!(signing_ceremony, messages, VerifyComm2, LocalSig3);

		// This account id will send an invalid signature
		let [bad_account_id] = signing_ceremony.select_account_ids();
		let invalid_sig3 = gen_dummy_local_sig(&mut signing_ceremony.rng);
		for message in messages.get_mut(&bad_account_id).unwrap().values_mut() {
			*message = invalid_sig3.clone();
		}

		let messages = signing_ceremony.run_stage::<VerifyLocalSig4, _, _>(messages).await;
		signing_ceremony.distribute_messages(messages).await;
		signing_ceremony
			.complete_with_error(&[bad_account_id], SigningFailureReason::InvalidSigShare)
			.await;
	}

	#[tokio::test]
	async fn should_report_on_deserialization_failure() {
		let (mut signing_ceremony, _) = new_signing_ceremony::<EthSigning>().await;

		let messages = signing_ceremony.request().await;
		let mut messages = run_stages!(signing_ceremony, messages, VerifyComm2, LocalSig3);

		// This account id will a message that cannot be deserialized
		let [bad_account_id] = signing_ceremony.select_account_ids();
		for message in messages.get_mut(&bad_account_id).unwrap().values_mut() {
			*message = DelayDeserialization::new(&"Not a valid LocalSig3");
		}

		let messages = signing_ceremony.run_stage::<VerifyLocalSig4, _, _>(messages).await;
		signing_ceremony.distribute_messages(messages).await;
		signing_ceremony
			.complete_with_error(&[bad_account_id], SigningFailureReason::DeserializationError)
			.await;
	}

	#[tokio::test]
	async fn should_report_on_invalid_number_of_local_signatures() {
		// A party that send the wrong number of local signatures (not matching
		// the number of payloads) should be reported

		let (mut signing_ceremony, _) = new_signing_ceremony::<BtcSigning>().await;

		let messages = signing_ceremony.request().await;
		let mut messages = run_stages!(signing_ceremony, messages, VerifyComm2, LocalSig3);

		// This account id will send an invalid signature
		let [bad_account_id] = signing_ceremony.select_account_ids();
		for message in messages.get_mut(&bad_account_id).unwrap().values_mut() {
			*message = {
				// Deserialize, add an (unexpected) extra local signature and serialize again:
				let mut sig = message.clone().deserialize().unwrap();
				sig.responses.push(sig.responses[0].clone());
				DelayDeserialization::new(&sig)
			}
		}

		let messages = signing_ceremony.run_stage::<VerifyLocalSig4, _, _>(messages).await;
		signing_ceremony.distribute_messages(messages).await;
		signing_ceremony
			.complete_with_error(&[bad_account_id], SigningFailureReason::InvalidNumberOfPayloads)
			.await;
	}
}

async fn test_sign_multiple_payloads<C: CryptoScheme>(payloads: &[C::SigningPayload]) {
	let mut rng = Rng::from_seed([0; 32]);
	let (key, key_data) =
		generate_key_data::<C>(BTreeSet::from_iter(ACCOUNT_IDS.iter().cloned()), &mut rng);

	let payloads_and_key = payloads
		.iter()
		.map(|payload| PayloadAndKeyData::new(payload.clone(), key.clone(), key_data.clone()))
		.collect();

	let mut signing_ceremony = SigningCeremonyRunner::<C>::new_with_all_signers(
		new_nodes(ACCOUNT_IDS.clone()),
		DEFAULT_SIGNING_CEREMONY_ID,
		payloads_and_key,
		rng,
	);

	let messages = signing_ceremony.request().await;
	let messages = run_stages!(
		signing_ceremony,
		messages,
		signing_data::VerifyComm2<C::Point>,
		signing_data::LocalSig3<C::Point>,
		signing_data::VerifyLocalSig4<C::Point>
	);
	signing_ceremony.distribute_messages(messages).await;
	let signature = signing_ceremony
		.complete()
		.await
		.into_iter()
		.next()
		.expect("should have exactly one signature");
	assert!(C::verify_signature(&signature, &key, &payloads[0]).is_ok());
}

#[tokio::test]
async fn should_sign_multiple_payloads() {
	// For now, only bitcoin can have multiple payloads. The other chains will fail the message size
	// check.

	let payloads = (1u8..=2).map(|i| bitcoin::SigningPayload([i; 32])).collect::<Vec<_>>();

	test_sign_multiple_payloads::<BtcSigning>(&payloads).await;
}

async fn should_sign_with_all_parties<C: CryptoScheme>() {
	// This seed ensures that the initially
	// generated key is incompatible to increase
	// test coverage
	for i in 0..10 {
		let key_seed = [i; 32];
		let nonce_seed = [11 * i; 32];
		let (key, key_data) = generate_key_data::<C>(
			BTreeSet::from_iter(ACCOUNT_IDS.iter().cloned()),
			&mut Rng::from_seed(key_seed),
		);

		let mut signing_ceremony = SigningCeremonyRunner::<C>::new_with_all_signers(
			new_nodes(ACCOUNT_IDS.clone()),
			DEFAULT_SIGNING_CEREMONY_ID,
			vec![PayloadAndKeyData::new(C::signing_payload_for_test(), key.clone(), key_data)],
			Rng::from_seed(nonce_seed),
		);

		let messages = signing_ceremony.request().await;
		let messages = run_stages!(
			signing_ceremony,
			messages,
			signing_data::VerifyComm2<C::Point>,
			signing_data::LocalSig3<C::Point>,
			signing_data::VerifyLocalSig4<C::Point>
		);
		signing_ceremony.distribute_messages(messages).await;
		let signature = signing_ceremony
			.complete()
			.await
			.into_iter()
			.next()
			.expect("should have exactly one signature");
		assert!(C::verify_signature(&signature, &key, &C::signing_payload_for_test()).is_ok());
	}
}

#[tokio::test]
async fn should_sign_with_different_keys() {
	// For now, only bitcoin can have multiple payloads. The other chains will fail the message size
	// check
	type C = BtcSigning;
	type Point = <C as CryptoScheme>::Point;

	let mut rng = Rng::from_seed([1; 32]);
	let account_ids = BTreeSet::from_iter(ACCOUNT_IDS.iter().cloned());

	// 1. Generate two different keys for the same set of validators.
	let (key_1, key_data_1) = generate_key_data::<C>(account_ids.clone(), &mut rng);
	let (key_2, key_data_2) = generate_key_data::<C>(account_ids.clone(), &mut rng);

	// Ensure we don't accidentally generate the same key (e.g. by using the same seed)
	assert_ne!(key_1, key_2);

	let mut signing_ceremony = SigningCeremonyRunner::<C>::new_with_all_signers(
		new_nodes(account_ids),
		DEFAULT_SIGNING_CEREMONY_ID,
		vec![
			PayloadAndKeyData::new(C::signing_payload_for_test(), key_1, key_data_1),
			PayloadAndKeyData::new(C::signing_payload_for_test(), key_2, key_data_2),
		],
		rng,
	);

	let messages = signing_ceremony.request().await;
	let messages = run_stages!(
		signing_ceremony,
		messages,
		signing_data::VerifyComm2<Point>,
		signing_data::LocalSig3<Point>,
		signing_data::VerifyLocalSig4<Point>
	);
	signing_ceremony.distribute_messages(messages).await;

	let signatures: Vec<_> = signing_ceremony.complete().await.into_iter().collect();

	assert_eq!(signatures.len(), 2);

	// Signatures should be correct w.r.t. corresponding keys:
	assert!(C::verify_signature(&signatures[0], &key_1, &C::signing_payload_for_test()).is_ok());
	assert!(C::verify_signature(&signatures[1], &key_2, &C::signing_payload_for_test()).is_ok());
}

#[tokio::test]
async fn should_sign_with_all_parties_eth() {
	should_sign_with_all_parties::<EthSigning>().await;
}

#[tokio::test]
async fn should_sign_with_all_parties_polkadot() {
	should_sign_with_all_parties::<PolkadotSigning>().await;
}

#[tokio::test]
async fn should_sign_with_all_parties_bitcoin() {
	should_sign_with_all_parties::<BtcSigning>().await;
}

mod timeout {

	use super::*;

	mod during_regular_stage {

		type SigningData = crate::client::signing::signing_data::SigningData<Point>;

		use super::*;

		mod should_recover_if_party_appears_offline_to_minority {

			use super::*;

			// The following tests cover:
			// If timeout during a regular (broadcast) stage, but the majority of nodes can agree on
			// all values, we proceed with the ceremony and use the data received by the majority.
			// If majority of nodes agree on a party timing out in the following broadcast
			// verification stage, the party gets reported

			#[tokio::test]
			async fn commitments_stage() {
				let (mut signing_ceremony, _) = new_signing_ceremony::<EthSigning>().await;

				let mut messages = signing_ceremony.request().await;

				let [non_sending_party_id, timed_out_party_id] =
					signing_ceremony.select_account_ids();

				messages.get_mut(&non_sending_party_id).unwrap().remove(&timed_out_party_id);

				signing_ceremony.distribute_messages(messages).await;

				// This node doesn't receive non_sending_party's message, so must timeout
				signing_ceremony
					.nodes
					.get_mut(&timed_out_party_id)
					.unwrap()
					.force_stage_timeout()
					.await;

				let messages =
					signing_ceremony.gather_outgoing_messages::<VerifyComm2, SigningData>().await;

				let messages = run_stages!(signing_ceremony, messages, LocalSig3, VerifyLocalSig4);
				signing_ceremony.distribute_messages(messages).await;
				signing_ceremony.complete().await;
			}

			#[tokio::test]
			async fn local_signatures_stage() {
				let (mut signing_ceremony, _) = new_signing_ceremony::<EthSigning>().await;

				let messages = signing_ceremony.request().await;

				let mut messages = run_stages!(signing_ceremony, messages, VerifyComm2, LocalSig3);

				let [non_sending_party_id, timed_out_party_id] =
					signing_ceremony.select_account_ids();

				messages.get_mut(&non_sending_party_id).unwrap().remove(&timed_out_party_id);

				signing_ceremony.distribute_messages(messages).await;

				// This node doesn't receive non_sending_party's message, so must timeout
				signing_ceremony
					.nodes
					.get_mut(&timed_out_party_id)
					.unwrap()
					.force_stage_timeout()
					.await;

				let messages = signing_ceremony
					.gather_outgoing_messages::<VerifyLocalSig4, SigningData>()
					.await;

				signing_ceremony.distribute_messages(messages).await;
				signing_ceremony.complete().await;
			}
		}
	}

	mod during_broadcast_verification_stage {

		use super::*;

		mod should_recover_if_agree_on_values {

			use super::*;

			// The following tests cover:
			// If timeout during a broadcast verification stage, and we have enough data, we can
			// recover

			#[tokio::test]
			async fn commitments_stage() {
				let (mut ceremony, _) = new_signing_ceremony::<EthSigning>().await;

				let [bad_node_id] = &ceremony.select_account_ids();

				let messages = ceremony.request().await;
				let messages = ceremony.run_stage::<VerifyComm2, _, _>(messages).await;

				let messages = ceremony
					.run_stage_with_non_sender::<LocalSig3, _, _>(messages, bad_node_id)
					.await;

				let messages = ceremony.run_stage::<VerifyLocalSig4, _, _>(messages).await;
				ceremony.distribute_messages(messages).await;
				ceremony.complete().await;
			}

			#[tokio::test]
			async fn local_signatures_stage() {
				let (mut ceremony, _) = new_signing_ceremony::<EthSigning>().await;

				let [bad_node_id] = &ceremony.select_account_ids();

				let messages = ceremony.request().await;
				let messages =
					run_stages!(ceremony, messages, VerifyComm2, LocalSig3, VerifyLocalSig4);

				ceremony.distribute_messages_with_non_sender(messages, bad_node_id).await;

				ceremony.complete().await;
			}
		}

		mod should_report_if_insufficient_messages {

			use super::*;
			// The following tests cover:
			// Timeout during both the broadcast & broadcast verification stages means that
			// we don't have enough data to recover:
			// The parties that timeout during the broadcast stage will be reported,
			// but the parties the timeout during the verification stage will not
			// because that would need another round of "voting" which can also timeout.

			#[tokio::test]
			async fn commitments_stage() {
				let (mut signing_ceremony, _) = new_signing_ceremony::<EthSigning>().await;

				// bad party 1 will timeout during a broadcast stage. It should be reported
				// bad party 2 will timeout during a broadcast verification stage. It won't get
				// reported.
				let [non_sending_party_id_1, non_sending_party_id_2] =
					signing_ceremony.select_account_ids();

				let messages = signing_ceremony.request().await;

				// bad party 1 times out here
				let messages = signing_ceremony
					.run_stage_with_non_sender::<VerifyComm2, _, _>(
						messages,
						&non_sending_party_id_1,
					)
					.await;

				// bad party 2 times out here (NB: They are different parties)
				signing_ceremony
					.distribute_messages_with_non_sender(messages, &non_sending_party_id_2)
					.await;

				signing_ceremony
					.complete_with_error(
						&[non_sending_party_id_1],
						SigningFailureReason::BroadcastFailure(
							BroadcastFailureReason::InsufficientMessages,
							SigningStageName::VerifyCommitmentsBroadcast2,
						),
					)
					.await
			}

			#[tokio::test]
			async fn local_signatures_stage() {
				let (mut signing_ceremony, _) = new_signing_ceremony::<EthSigning>().await;

				// bad party 1 will timeout during a broadcast stage. It should be reported
				// bad party 2 will timeout during a broadcast verification stage. It won't get
				// reported.
				let [non_sending_party_id_1, non_sending_party_id_2] =
					signing_ceremony.select_account_ids();

				let messages = signing_ceremony.request().await;

				let messages = run_stages!(signing_ceremony, messages, VerifyComm2, LocalSig3);

				// bad party 1 times out here
				let messages = signing_ceremony
					.run_stage_with_non_sender::<VerifyLocalSig4, _, _>(
						messages,
						&non_sending_party_id_1,
					)
					.await;

				// bad party 2 times out here (NB: They are different parties)
				signing_ceremony
					.distribute_messages_with_non_sender(messages, &non_sending_party_id_2)
					.await;

				signing_ceremony
					.complete_with_error(
						&[non_sending_party_id_1],
						SigningFailureReason::BroadcastFailure(
							BroadcastFailureReason::InsufficientMessages,
							SigningStageName::VerifyLocalSigsBroadcastStage4,
						),
					)
					.await
			}
		}
	}
}
