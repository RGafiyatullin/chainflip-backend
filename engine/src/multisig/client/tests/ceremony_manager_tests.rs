use std::collections::BTreeSet;

use super::{
    helpers::gen_invalid_keygen_stage_2_state,
    keygen_data_tests::gen_keygen_data_verify_hash_comm2, *,
};
use crate::{
    constants::CEREMONY_ID_WINDOW,
    logging::test_utils::new_test_logger,
    multisig::{
        client::{
            self,
            ceremony_manager::CeremonyManager,
            common::SigningFailureReason,
            keygen::{get_key_data_for_test, KeygenData},
            CeremonyFailureReason, MultisigData,
        },
        crypto::{CryptoScheme, Rng},
        eth::EthSigning,
        tests::fixtures::MESSAGE_HASH,
    },
};
use client::MultisigMessage;
use rand_legacy::SeedableRng;
use sp_runtime::AccountId32;
use tokio::sync::oneshot;
use utilities::{assert_panics, threshold_from_share_count};

/// Run on_request_to_sign on a ceremony manager, using a junk key and default ceremony id and data.
fn run_on_request_to_sign<C: CryptoScheme>(
    ceremony_manager: &mut CeremonyManager<C>,
    participants: Vec<sp_runtime::AccountId32>,
) -> oneshot::Receiver<
    Result<
        <C as CryptoScheme>::Signature,
        (
            BTreeSet<AccountId32>,
            CeremonyFailureReason<SigningFailureReason>,
        ),
    >,
> {
    let (result_sender, result_receiver) = oneshot::channel();
    ceremony_manager.on_request_to_sign(
        DEFAULT_SIGNING_CEREMONY_ID,
        participants,
        MESSAGE_HASH.clone(),
        get_key_data_for_test(&ACCOUNT_IDS),
        Rng::from_seed(DEFAULT_SIGNING_SEED),
        result_sender,
    );
    result_receiver
}

/// Create an Eth ceremony manager with default latest ceremony id and dropped p2p receiver.
fn new_ceremony_manager_for_test(our_account_id: AccountId) -> CeremonyManager<EthSigning> {
    CeremonyManager::<EthSigning>::new(
        our_account_id,
        tokio::sync::mpsc::unbounded_channel().0,
        INITIAL_LATEST_CEREMONY_ID,
        &new_test_logger(),
    )
}

#[tokio::test]
async fn should_panic_keygen_request_if_not_participating() {
    let non_participating_id = AccountId::new([0; 32]);
    assert!(!ACCOUNT_IDS.contains(&non_participating_id));

    // Create a new ceremony manager with the non_participating_id
    let mut ceremony_manager = new_ceremony_manager_for_test(non_participating_id);

    // Send a keygen request where participants doesn't include non_participating_id
    let (result_sender, _result_receiver) = oneshot::channel();
    assert_panics!(ceremony_manager.on_keygen_request(
        DEFAULT_KEYGEN_CEREMONY_ID,
        ACCOUNT_IDS.clone(),
        Rng::from_seed(DEFAULT_KEYGEN_SEED),
        result_sender,
    ));
}

#[tokio::test]
async fn should_panic_rts_if_not_participating() {
    let non_participating_id = AccountId::new([0; 32]);
    assert!(!ACCOUNT_IDS.contains(&non_participating_id));

    // Create a new ceremony manager with the non_participating_id
    let mut ceremony_manager = new_ceremony_manager_for_test(non_participating_id);

    // Send a signing request where participants doesn't include non_participating_id
    assert_panics!(run_on_request_to_sign(
        &mut ceremony_manager,
        ACCOUNT_IDS.clone()
    ));
}

#[tokio::test]
async fn should_ignore_keygen_request_with_duplicate_signer() {
    // Create a list of participants with a duplicate id
    let mut participants = ACCOUNT_IDS.clone();
    participants[1] = participants[2].clone();

    let mut ceremony_manager = new_ceremony_manager_for_test(ACCOUNT_IDS[0].clone());

    // Send a keygen request with the duplicate id
    let (result_sender, mut result_receiver) = oneshot::channel();
    ceremony_manager.on_keygen_request(
        DEFAULT_KEYGEN_CEREMONY_ID,
        participants,
        Rng::from_seed(DEFAULT_KEYGEN_SEED),
        result_sender,
    );

    // Receive the InvalidParticipants error result
    assert_eq!(
        result_receiver
            .try_recv()
            .expect("Failed to receive ceremony result"),
        Err((
            BTreeSet::default(),
            CeremonyFailureReason::InvalidParticipants
        ))
    );
}

#[tokio::test]
async fn should_ignore_rts_with_duplicate_signer() {
    // Create a list of signers with a duplicate id
    let mut participants = ACCOUNT_IDS.clone();
    participants[1] = participants[2].clone();

    let mut ceremony_manager = new_ceremony_manager_for_test(ACCOUNT_IDS[0].clone());

    // Send a signing request with the duplicate id
    let mut result_receiver = run_on_request_to_sign(&mut ceremony_manager, participants);

    // Receive the InvalidParticipants error result
    assert_eq!(
        result_receiver
            .try_recv()
            .expect("Failed to receive ceremony result"),
        Err((
            BTreeSet::default(),
            CeremonyFailureReason::InvalidParticipants
        ))
    );
}

#[tokio::test]
async fn should_ignore_rts_with_insufficient_number_of_signers() {
    // Create a list of signers that is equal to the threshold (not enough to generate a signature)
    let threshold = threshold_from_share_count(ACCOUNT_IDS.len() as u32) as usize;
    let not_enough_participants = ACCOUNT_IDS[0..threshold].to_vec();

    let mut ceremony_manager = new_ceremony_manager_for_test(ACCOUNT_IDS[0].clone());

    // Send a signing request with not enough participants
    let mut result_receiver =
        run_on_request_to_sign(&mut ceremony_manager, not_enough_participants);

    // Receive the NotEnoughSigners error result
    assert_eq!(
        result_receiver
            .try_recv()
            .expect("Failed to receive ceremony result"),
        Err((
            BTreeSet::default(),
            CeremonyFailureReason::Other(SigningFailureReason::NotEnoughSigners),
        ))
    );
}

#[tokio::test]
async fn should_ignore_rts_with_unknown_signer_id() {
    let our_account_id_idx = 0;
    let unknown_signer_idx = 1;
    assert_ne!(
        our_account_id_idx, unknown_signer_idx,
        "The unknown id must not be our own id or the test is invalid"
    );

    // Create a new ceremony manager with an account id that is in ACCOUNT_IDS
    let mut ceremony_manager =
        new_ceremony_manager_for_test(ACCOUNT_IDS[our_account_id_idx].clone());

    // Replace one of the signers with an unknown id
    let unknown_signer_id = AccountId::new([0; 32]);
    assert!(!ACCOUNT_IDS.contains(&unknown_signer_id));
    let mut participants = ACCOUNT_IDS.clone();
    participants[unknown_signer_idx] = unknown_signer_id;

    // Send a signing request with the modified participants
    let mut result_receiver = run_on_request_to_sign(&mut ceremony_manager, participants);

    // Receive the InvalidParticipants error result
    assert_eq!(
        result_receiver
            .try_recv()
            .expect("Failed to receive ceremony result"),
        Err((
            BTreeSet::default(),
            CeremonyFailureReason::InvalidParticipants,
        ))
    );
}

#[tokio::test]
async fn should_ignore_stage_data_with_incorrect_size() {
    let logger = new_test_logger();
    let rng = Rng::from_seed(DEFAULT_KEYGEN_SEED);
    let ceremony_id = DEFAULT_KEYGEN_CEREMONY_ID;
    let num_of_participants = ACCOUNT_IDS.len() as u32;

    // This test only works on message stage data that can have incorrect size (ie. not first stage),
    // so we must create a stage 2 state and add it to the ceremony managers keygen states,
    // allowing us to process a stage 2 message.
    let mut stage_2_state = gen_invalid_keygen_stage_2_state::<<EthSigning as CryptoScheme>::Point>(
        ceremony_id,
        &ACCOUNT_IDS[..],
        rng,
        logger.clone(),
    );

    // Built a stage 2 message that has the incorrect number of elements
    let stage_2_data = gen_keygen_data_verify_hash_comm2(num_of_participants + 1);

    // Process the bad message and it should get rejected
    assert_eq!(
        stage_2_state
            .process_or_delay_message(ACCOUNT_IDS[0].clone(), stage_2_data)
            .await,
        None
    );

    // Check that the bad message was ignored, so the stage is still awaiting all num_of_participants messages.
    assert_eq!(
        stage_2_state.get_awaited_parties_count(),
        Some(num_of_participants)
    );
}

#[test]
#[ignore = "temporarily disabled - see issue #1972"]
fn should_not_create_unauthorized_ceremony_with_invalid_ceremony_id() {
    let latest_ceremony_id = 1; // Invalid, because the CeremonyManager starts with this value as the latest
    let past_ceremony_id = latest_ceremony_id - 1; // Invalid, because it was used in the past
    let future_ceremony_id = latest_ceremony_id + CEREMONY_ID_WINDOW; // Valid, because its within the window
    let future_ceremony_id_too_large = latest_ceremony_id + CEREMONY_ID_WINDOW + 1; // Invalid, because its too far in the future

    // Junk stage 1 data to use for the test
    let stage_1_data = MultisigData::Keygen(KeygenData::HashComm1(client::keygen::HashComm1(
        sp_core::H256::default(),
    )));

    // Create a new ceremony manager and set the latest_ceremony_id
    let mut ceremony_manager = CeremonyManager::<EthSigning>::new(
        ACCOUNT_IDS[0].clone(),
        tokio::sync::mpsc::unbounded_channel().0,
        latest_ceremony_id,
        &new_test_logger(),
    );

    // Process a stage 1 message with a ceremony id that is in the past
    ceremony_manager.process_p2p_message(
        ACCOUNT_IDS[0].clone(),
        MultisigMessage {
            ceremony_id: past_ceremony_id,
            data: stage_1_data.clone(),
        },
    );

    // Process a stage 1 message with a ceremony id that is too far in the future
    ceremony_manager.process_p2p_message(
        ACCOUNT_IDS[0].clone(),
        MultisigMessage {
            ceremony_id: future_ceremony_id_too_large,
            data: stage_1_data.clone(),
        },
    );

    // Check that the messages were ignored and no unauthorised ceremonies were created
    assert_eq!(ceremony_manager.get_keygen_states_len(), 0);

    // Process a stage 1 message with a ceremony id that in the future but still within the window
    ceremony_manager.process_p2p_message(
        ACCOUNT_IDS[0].clone(),
        MultisigMessage {
            ceremony_id: future_ceremony_id,
            data: stage_1_data,
        },
    );

    // Check that the message was not ignored and unauthorised ceremony was created
    assert_eq!(ceremony_manager.get_keygen_states_len(), 1);
}
