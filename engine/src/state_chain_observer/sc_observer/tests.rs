use std::{collections::BTreeSet, sync::Arc};

use crate::{
	btc::rpc::MockBtcRpcApi,
	state_chain_observer::client::{extrinsic_api, StreamCache},
};
use cf_chains::{
	eth::{SchnorrVerificationComponents, Transaction},
	ChainCrypto,
};
use cf_primitives::{AccountRole, GENESIS_EPOCH};
use frame_system::Phase;
use futures::{FutureExt, StreamExt};
use mockall::predicate::eq;
use multisig::{eth::EvmCryptoScheme, ChainSigning, SignatureToThresholdSignature};
use pallet_cf_broadcast::BroadcastAttemptId;
use sp_runtime::{AccountId32, Digest};

use crate::eth::ethers_rpc::MockEthersRpcApi;
use sp_core::H256;
use state_chain_runtime::{
	AccountId, BitcoinInstance, EthereumInstance, Header, PolkadotInstance, Runtime, RuntimeCall,
	RuntimeEvent,
};
use utilities::MakeCachedStream;

use crate::{
	btc::BtcBroadcaster,
	dot::{rpc::MockDotRpcApi, DotBroadcaster},
	eth::broadcaster::EthBroadcaster,
	settings::Settings,
	state_chain_observer::{client::mocks::MockStateChainClient, sc_observer},
};
use multisig::{
	client::{KeygenFailureReason, MockMultisigClientApi, SigningFailureReason},
	eth::EthSigning,
	CryptoScheme, KeyId,
};
use utilities::task_scope::task_scope;

use super::crypto_compat::CryptoCompat;

fn test_header(number: u32) -> Header {
	Header {
		number,
		parent_hash: H256::default(),
		state_root: H256::default(),
		extrinsics_root: H256::default(),
		digest: Digest { logs: Vec::new() },
	}
}

const MOCK_ETH_TRANSACTION_OUT_ID: SchnorrVerificationComponents =
	SchnorrVerificationComponents { s: [0; 32], k_times_g_address: [1; 20] };

async fn start_sc_observer<
	BlockStream: crate::state_chain_observer::client::StateChainStreamApi,
>(
	state_chain_client: MockStateChainClient,
	sc_block_stream: BlockStream,
	eth_rpc: MockEthersRpcApi,
) {
	let (account_peer_mapping_change_sender, _account_peer_mapping_change_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	sc_observer::start(
		Arc::new(state_chain_client),
		sc_block_stream,
		EthBroadcaster::new_test(eth_rpc),
		DotBroadcaster::new(MockDotRpcApi::new()),
		BtcBroadcaster::new(MockBtcRpcApi::new()),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		account_peer_mapping_change_sender,
	)
	.await
	.unwrap_err();
}

// TODO: We should test that this works for historical epochs too. We should be able to sign for
// historical epochs we were a part of
#[tokio::test]
async fn only_encodes_and_signs_when_specified() {
	let initial_block_hash = H256::default();
	let account_id = AccountId::new([0; 32]);

	let mut state_chain_client = MockStateChainClient::new();

	state_chain_client.expect_account_id().once().return_once({
		let account_id = account_id.clone();
		|| account_id
	});

	let block_header = test_header(21);
	let sc_block_stream = tokio_stream::iter([block_header.clone()])
		.map(|block_header| (block_header.hash(), block_header))
		.make_cached(
			StreamCache { block_hash: initial_block_hash, block_number: 20 },
			|(block_hash, block_header): &(
				state_chain_runtime::Hash,
				state_chain_runtime::Header,
			)| StreamCache { block_hash: *block_hash, block_number: block_header.number },
		);

	state_chain_client
		.expect_storage_value::<frame_system::Events<Runtime>>()
		.with(eq(block_header.hash()))
		.once()
		.return_once(move |_| {
			Ok(vec![
				Box::new(frame_system::EventRecord {
					phase: Phase::ApplyExtrinsic(0),
					event: RuntimeEvent::EthereumBroadcaster(
						pallet_cf_broadcast::Event::TransactionBroadcastRequest {
							broadcast_attempt_id: BroadcastAttemptId::default(),
							nominee: account_id,
							transaction_payload: Transaction::default(),
							transaction_out_id: MOCK_ETH_TRANSACTION_OUT_ID,
						},
					),
					topics: vec![H256::default()],
				}),
				Box::new(frame_system::EventRecord {
					phase: Phase::ApplyExtrinsic(1),
					event: RuntimeEvent::EthereumBroadcaster(
						pallet_cf_broadcast::Event::TransactionBroadcastRequest {
							broadcast_attempt_id: BroadcastAttemptId::default(),
							nominee: AccountId32::new([1; 32]), // NOT OUR ACCOUNT ID
							transaction_payload: Transaction::default(),
							transaction_out_id: MOCK_ETH_TRANSACTION_OUT_ID,
						},
					),
					topics: vec![H256::default()],
				}),
			])
		});

	let mut ethers_rpc_mock = MockEthersRpcApi::new();
	// when we are selected to sign we must estimate gas and sign
	// NB: We only do this once, since we are only selected to sign once
	ethers_rpc_mock
		.expect_estimate_gas()
		.once()
		.returning(|_| Ok(ethers::types::U256::from(100_000)));

	ethers_rpc_mock.expect_send_transaction().once().return_once(|tx| {
		// return some hash
		Ok(tx.sighash())
	});

	start_sc_observer(state_chain_client, sc_block_stream, ethers_rpc_mock).await;
}

// TODO: Test that when we return None for polkadot vault
// witnessing isn't started for dot, but is started for ETH

/// Test all 3 cases of handling a signing request: not participating, failure, and success.
async fn should_handle_signing_request<C, I>()
where
	C: CryptoScheme + Send + Sync,
	I: 'static + Send + Sync,

	Runtime: pallet_cf_threshold_signature::Config<I>,
	RuntimeCall:
		std::convert::From<pallet_cf_threshold_signature::Call<Runtime, I>>,
	<<Runtime as pallet_cf_threshold_signature::Config<I>>::TargetChain as
ChainCrypto>::ThresholdSignature: std::convert::From<<C as CryptoScheme>::Signature>,
	Vec<C::Signature>: SignatureToThresholdSignature<
		<Runtime as pallet_cf_threshold_signature::Config<I>>::TargetChain

	>,
{
	let key_id = KeyId::new(1, [0u8; 32]);
	let payload = C::signing_payload_for_test();
	let our_account_id = AccountId32::new([0; 32]);
	let not_our_account_id = AccountId32::new([1u8; 32]);
	assert_ne!(our_account_id, not_our_account_id);

	let mut state_chain_client = MockStateChainClient::new();
	let mut multisig_client = MockMultisigClientApi::<C>::new();

	// All 3 signing requests will ask for the account id
	state_chain_client
		.expect_account_id()
		.times(3)
		.return_const(our_account_id.clone());

	// ceremony_id_1 is a non-participating ceremony and should update the latest ceremony id
	let ceremony_id_1 = 1;
	multisig_client
		.expect_update_latest_ceremony_id()
		.with(eq(ceremony_id_1))
		.once()
		.returning(|_| ());

	// ceremony_id_2 is a failure and should submit a signed extrinsic
	let ceremony_id_2 = ceremony_id_1 + 1;
	multisig_client
		.expect_initiate_signing()
		.with(
			eq(ceremony_id_2),
			eq(BTreeSet::from_iter([our_account_id.clone()])),
			eq(vec![(key_id.clone(), payload.clone())]),
		)
		.once()
		.return_once(|_, _, _| {
			futures::future::ready(Err((
				BTreeSet::new(),
				SigningFailureReason::InvalidParticipants,
			)))
			.boxed()
		});
	state_chain_client
		.expect_submit_signed_extrinsic::<pallet_cf_threshold_signature::Call<Runtime, I>>()
		.with(eq(pallet_cf_threshold_signature::Call::<Runtime, I>::report_signature_failed {
			ceremony_id: ceremony_id_2,
			offenders: BTreeSet::default(),
		}))
		.once()
		.return_once(|_| (H256::default(), extrinsic_api::signed::MockUntilFinalized::new()));

	// ceremony_id_3 is a success and should submit an unsigned extrinsic
	let ceremony_id_3 = ceremony_id_2 + 1;
	let signatures = vec![C::signature_for_test()];
	let signatures_clone = signatures.clone();
	multisig_client
		.expect_initiate_signing()
		.with(
			eq(ceremony_id_3),
			eq(BTreeSet::from_iter([our_account_id.clone()])),
			eq(vec![(key_id.clone(), payload.clone())]),
		)
		.once()
		.return_once(move |_, _, _| futures::future::ready(Ok(signatures_clone)).boxed());
	state_chain_client
		.expect_submit_unsigned_extrinsic()
		.with(eq(pallet_cf_threshold_signature::Call::<Runtime, I>::signature_success {
			ceremony_id: ceremony_id_3,
			signature: signatures.to_threshold_signature(),
		}))
		.once()
		.return_once(|_: pallet_cf_threshold_signature::Call<Runtime, I>| H256::default());

	let state_chain_client = Arc::new(state_chain_client);
	task_scope(|scope| {
		async {
			// Handle a signing request that we are not participating in
			sc_observer::handle_signing_request::<_, _, C, I>(
				scope,
				&multisig_client,
				state_chain_client.clone(),
				ceremony_id_1,
				BTreeSet::from_iter([not_our_account_id.clone()]),
				vec![(key_id.clone(), payload.clone())],
			)
			.await;

			// Handle a signing request that we are participating in.
			// This one will return an error.
			sc_observer::handle_signing_request::<_, _, C, I>(
				scope,
				&multisig_client,
				state_chain_client.clone(),
				ceremony_id_2,
				BTreeSet::from_iter([our_account_id.clone()]),
				vec![(key_id.clone(), payload.clone())],
			)
			.await;

			// Handle another signing request that we are participating in.
			// This one will return success.
			sc_observer::handle_signing_request::<_, _, C, I>(
				scope,
				&multisig_client,
				state_chain_client.clone(),
				ceremony_id_3,
				BTreeSet::from_iter([our_account_id]),
				vec![(key_id, payload)],
			)
			.await;

			Ok(())
		}
		.boxed()
	})
	.await
	.unwrap();
}

// Test that the ceremony requests are calling the correct MultisigClientApi functions
// depending on whether we are participating in the ceremony or not.
#[tokio::test]
async fn should_handle_signing_request_eth() {
	should_handle_signing_request::<EvmCryptoScheme, EthereumInstance>().await;
}

mod dot_signing {

	use multisig::polkadot::PolkadotCryptoScheme;

	use super::*;
	use PolkadotInstance;

	#[tokio::test]
	async fn should_handle_signing_request_dot() {
		should_handle_signing_request::<PolkadotCryptoScheme, PolkadotInstance>().await;
	}
}

async fn should_handle_keygen_request<C, I>()
where
	C: ChainSigning<Chain = <Runtime as pallet_cf_vaults::Config<I>>::Chain> + Send + Sync,
	I: CryptoCompat<C, C::Chain> + 'static + Send + Sync,
	Runtime: pallet_cf_vaults::Config<I>,
	RuntimeCall: std::convert::From<pallet_cf_vaults::Call<Runtime, I>>,
{
	let first_ceremony_id = 1;
	let our_account_id = AccountId32::new([0; 32]);
	let not_our_account_id = AccountId32::new([1u8; 32]);
	assert_ne!(our_account_id, not_our_account_id);

	let mut state_chain_client = MockStateChainClient::new();
	state_chain_client
		.expect_account_id()
		.times(2)
		.return_const(our_account_id.clone());
	state_chain_client
		.expect_submit_signed_extrinsic::<pallet_cf_vaults::Call<Runtime, I>>()
		.once()
		.return_once(|_| (H256::default(), extrinsic_api::signed::MockUntilFinalized::new()));
	let state_chain_client = Arc::new(state_chain_client);

	let mut multisig_client = MockMultisigClientApi::<C::CryptoScheme>::new();
	multisig_client
		.expect_update_latest_ceremony_id()
		.with(eq(first_ceremony_id))
		.once()
		.return_once(|_| ());

	let next_ceremony_id = first_ceremony_id + 1;
	// Set up the mock api to expect the keygen and sign calls for the ceremonies we are
	// participating in. It doesn't matter what failure reasons they return.
	multisig_client
		.expect_initiate_keygen()
		.with(
			eq(next_ceremony_id),
			eq(GENESIS_EPOCH),
			eq(BTreeSet::from_iter([our_account_id.clone()])),
		)
		.once()
		.return_once(|_, _, _| {
			futures::future::ready(Err((BTreeSet::new(), KeygenFailureReason::InvalidParticipants)))
				.boxed()
		});

	task_scope(|scope| {
		async {
			// Handle a keygen request that we are not participating in
			sc_observer::handle_keygen_request::<_, _, _, I>(
				scope,
				&multisig_client,
				state_chain_client.clone(),
				first_ceremony_id,
				GENESIS_EPOCH,
				BTreeSet::from_iter([not_our_account_id.clone()]),
			)
			.await;

			// Handle a keygen request that we are participating in
			sc_observer::handle_keygen_request::<_, _, _, I>(
				scope,
				&multisig_client,
				state_chain_client.clone(),
				next_ceremony_id,
				GENESIS_EPOCH,
				BTreeSet::from_iter([our_account_id]),
			)
			.await;
			Ok(())
		}
		.boxed()
	})
	.await
	.unwrap();
}

#[tokio::test]
async fn should_handle_keygen_request_eth() {
	should_handle_keygen_request::<EthSigning, EthereumInstance>().await;
}

mod dot_keygen {
	use multisig::polkadot::PolkadotSigning;

	use super::*;
	use PolkadotInstance;
	#[tokio::test]
	async fn should_handle_keygen_request_dot() {
		should_handle_keygen_request::<PolkadotSigning, PolkadotInstance>().await;
	}
}

#[tokio::test]
async fn should_handle_key_handover_request()
where
	Runtime: pallet_cf_vaults::Config<BitcoinInstance>,
	RuntimeCall: std::convert::From<pallet_cf_vaults::Call<Runtime, BitcoinInstance>>,
{
	use multisig::bitcoin::BtcCryptoScheme;

	let first_ceremony_id = 1;
	let our_account_id = AccountId32::new([0; 32]);
	let not_our_account_id = AccountId32::new([1u8; 32]);
	assert_ne!(our_account_id, not_our_account_id);

	let mut state_chain_client = MockStateChainClient::new();
	let mut multisig_client = MockMultisigClientApi::<BtcCryptoScheme>::new();

	// Both requests will ask for the account id
	state_chain_client
		.expect_account_id()
		.times(2)
		.return_const(our_account_id.clone());

	// The first ceremony is a non-participating ceremony so it should update the latest ceremony id
	multisig_client
		.expect_update_latest_ceremony_id()
		.with(eq(first_ceremony_id))
		.once()
		.return_once(|_| ());

	// The second ceremony is a failure and should submit a signed extrinsic
	let next_ceremony_id = first_ceremony_id + 1;
	let key_to_share = cf_chains::btc::AggKey::default();
	multisig_client
		.expect_initiate_key_handover()
		.with(
			eq(next_ceremony_id),
			eq(KeyId::new(GENESIS_EPOCH, key_to_share.current)),
			eq(GENESIS_EPOCH + 1),
			eq(BTreeSet::from_iter([our_account_id.clone()])),
			eq(BTreeSet::from_iter([our_account_id.clone()])),
		)
		.once()
		.return_once(|_, _, _, _, _| {
			futures::future::ready(Err((BTreeSet::new(), KeygenFailureReason::InvalidParticipants)))
				.boxed()
		});
	state_chain_client
		.expect_submit_signed_extrinsic::<pallet_cf_vaults::Call<Runtime, BitcoinInstance>>()
		.once()
		.return_once(|_| (H256::default(), extrinsic_api::signed::MockUntilFinalized::new()));

	let state_chain_client = Arc::new(state_chain_client);
	task_scope(|scope| {
		async {
			// Handle the key handover request that we are not participating in
			sc_observer::handle_key_handover_request::<_, _>(
				scope,
				&multisig_client,
				state_chain_client.clone(),
				first_ceremony_id,
				GENESIS_EPOCH,
				GENESIS_EPOCH + 1,
				BTreeSet::from_iter([not_our_account_id.clone()]),
				BTreeSet::from_iter([not_our_account_id.clone()]),
				key_to_share,
				Default::default(),
			)
			.await;

			// Handle the key handover request that we are participating in
			sc_observer::handle_key_handover_request::<_, _>(
				scope,
				&multisig_client,
				state_chain_client.clone(),
				next_ceremony_id,
				GENESIS_EPOCH,
				GENESIS_EPOCH + 1,
				BTreeSet::from_iter([our_account_id.clone()]),
				BTreeSet::from_iter([our_account_id.clone()]),
				key_to_share,
				Default::default(),
			)
			.await;
			Ok(())
		}
		.boxed()
	})
	.await
	.unwrap();
}

#[tokio::test]
#[ignore = "runs forever, useful for testing without having to start the whole CFE"]
async fn run_the_sc_observer() {
	task_scope(|scope| {
		async {
			let settings = Settings::new_test().unwrap();

			let (sc_block_stream, state_chain_client) =
				crate::state_chain_observer::client::StateChainClient::connect_with_account(
					scope,
					&settings.state_chain.ws_endpoint,
					&settings.state_chain.signing_key_file,
					AccountRole::None,
					false,
				)
				.await
				.unwrap();

			let (account_peer_mapping_change_sender, _account_peer_mapping_change_receiver) =
				tokio::sync::mpsc::unbounded_channel();

			sc_observer::start(
				state_chain_client,
				sc_block_stream,
				EthBroadcaster::new(MockEthersRpcApi::new()),
				DotBroadcaster::new(MockDotRpcApi::new()),
				BtcBroadcaster::new(MockBtcRpcApi::new()),
				MockMultisigClientApi::new(),
				MockMultisigClientApi::new(),
				MockMultisigClientApi::new(),
				account_peer_mapping_change_sender,
			)
			.await
			.unwrap_err();

			Ok(())
		}
		.boxed()
	})
	.await
	.unwrap();
}
