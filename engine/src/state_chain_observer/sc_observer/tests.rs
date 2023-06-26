use std::{collections::BTreeSet, sync::Arc};

use crate::{
	btc::rpc::MockBtcRpcApi,
	state_chain_observer::client::{extrinsic_api, StreamCache},
};
use cf_chains::{
	dot::PolkadotAccountId,
	eth::{Ethereum, SchnorrVerificationComponents, Transaction},
	ChainCrypto,
};
use cf_primitives::{AccountRole, GENESIS_EPOCH};
use frame_system::Phase;
use futures::{FutureExt, StreamExt};
use mockall::predicate::{self, eq};
use multisig::SignatureToThresholdSignature;
use pallet_cf_broadcast::BroadcastAttemptId;
use pallet_cf_vaults::Vault;
use sp_runtime::{AccountId32, Digest};

use crate::eth::ethers_rpc::MockEthersRpcApi;
use sp_core::H256;
use state_chain_runtime::{AccountId, CfeSettings, EthereumInstance, Header};
use tokio::sync::watch;
use utilities::{assert_future_panics, MakeCachedStream};

use crate::{
	btc::BtcBroadcaster,
	dot::{rpc::MockDotRpcApi, DotBroadcaster},
	eth::{broadcaster::EthBroadcaster, ethers_rpc::EthersRpcClient},
	settings::Settings,
	state_chain_observer::{client::mocks::MockStateChainClient, sc_observer},
	witnesser::EpochStart,
};
use multisig::{
	client::{KeygenFailureReason, MockMultisigClientApi, SigningFailureReason},
	eth::EthSigning,
	CryptoScheme, KeyId,
};
use utilities::task_scope::task_scope;

use super::{crypto_compat::CryptoCompat, EthAddressToMonitorSender};

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

#[tokio::test]
async fn starts_witnessing_when_current_authority() {
	let initial_epoch = 3;
	let initial_epoch_from_block_eth = 30;
	let initial_block_hash = H256::default();
	let account_id = AccountId::new([0; 32]);

	let mut state_chain_client = MockStateChainClient::new();

	state_chain_client.expect_account_id().return_once({
		let account_id = account_id.clone();
		|| account_id
	});

	state_chain_client.
expect_storage_map_entry::<pallet_cf_validator::HistoricalActiveEpochs<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash), eq(account_id))
		.once()
		.return_once(move |_, _| Ok(vec![initial_epoch]));
	state_chain_client
		.expect_storage_value::<pallet_cf_validator::CurrentEpoch<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(move |_| Ok(initial_epoch));
	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: initial_epoch_from_block_eth,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 80 }))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 98 }))
		});

	state_chain_client
			.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
				state_chain_runtime::Runtime,
			>>()
			.with(eq(initial_block_hash))
			.once()
			.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	// No blocks in the stream
	let sc_block_stream = tokio_stream::iter(vec![]).make_cached(
		StreamCache { block_hash: Default::default(), block_number: Default::default() },
		|(block_hash, block_header): &(state_chain_runtime::Hash, state_chain_runtime::Header)| {
			StreamCache { block_hash: *block_hash, block_number: block_header.number }
		},
	);

	let (account_peer_mapping_change_sender, _account_peer_mapping_change_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (epoch_start_sender, epoch_start_receiver) = async_broadcast::broadcast(10);

	let (cfe_settings_update_sender, _) = watch::channel::<CfeSettings>(CfeSettings::default());

	let (eth_monitor_command_sender, _eth_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_flip_command_sender, _eth_monitor_flip_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_usdc_command_sender, _eth_monitor_usdc_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_epoch_start_sender, _dot_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (dot_monitor_command_sender, _dot_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_monitor_signature_sender, _dot_monitor_signature_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_epoch_start_sender, _btc_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (btc_address_monitor_sender, _btc_address_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_tx_hash_monitor_sender, _btc_tx_hash_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	sc_observer::start(
		Arc::new(state_chain_client),
		sc_block_stream,
		EthBroadcaster::new_test(MockEthersRpcApi::new()),
		DotBroadcaster::new(MockDotRpcApi::new()),
		BtcBroadcaster::new(MockBtcRpcApi::new()),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		account_peer_mapping_change_sender,
		epoch_start_sender,
		EthAddressToMonitorSender {
			eth: eth_monitor_command_sender,
			flip: eth_monitor_flip_command_sender,
			usdc: eth_monitor_usdc_command_sender,
		},
		dot_epoch_start_sender,
		dot_monitor_command_sender,
		dot_monitor_signature_sender,
		btc_epoch_start_sender,
		btc_address_monitor_sender,
		btc_tx_hash_monitor_sender,
		cfe_settings_update_sender,
	)
	.await
	.unwrap_err();
	assert_eq!(
		epoch_start_receiver.collect::<Vec<_>>().await,
		vec![EpochStart::<Ethereum> {
			epoch_index: initial_epoch,
			block_number: initial_epoch_from_block_eth,
			current: true,
			participant: true,
			data: ()
		}]
	);
}

#[tokio::test]
async fn starts_witnessing_when_historic_on_startup() {
	let active_epoch = 3;
	let active_epoch_from_block_eth = 30;
	let current_epoch = 4;
	let current_epoch_from_block_eth = 40;

	let current_epoch_from_block_dot = 80;

	let current_epoch_from_block_btc = 100;

	let initial_block_hash = H256::default();
	let account_id = AccountId::new([0; 32]);

	let mut state_chain_client = MockStateChainClient::new();

	state_chain_client.expect_account_id().once().return_once({
		let account_id = account_id.clone();
		|| account_id
	});

	state_chain_client.
expect_storage_map_entry::<pallet_cf_validator::HistoricalActiveEpochs<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash), eq(account_id))
		.once()
		.return_once(move |_, _| Ok(vec![active_epoch]));
	state_chain_client
		.expect_storage_value::<pallet_cf_validator::CurrentEpoch<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(move |_| Ok(current_epoch));
	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(initial_block_hash), eq(active_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: active_epoch_from_block_eth,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(initial_block_hash), eq(active_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: current_epoch_from_block_dot,
			}))
		});

	state_chain_client
		.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
			state_chain_runtime::Runtime,
		>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(initial_block_hash), eq(active_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 98 }))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(initial_block_hash), eq(current_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: current_epoch_from_block_eth,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(initial_block_hash), eq(current_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: current_epoch_from_block_dot,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(initial_block_hash), eq(current_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: current_epoch_from_block_btc
})) 		});

	state_chain_client
		.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
			state_chain_runtime::Runtime,
		>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	// No blocks in the stream
	let sc_block_stream = tokio_stream::iter(vec![]).make_cached(
		StreamCache { block_hash: initial_block_hash, block_number: 20 },
		|(block_hash, block_header): &(state_chain_runtime::Hash, state_chain_runtime::Header)| {
			StreamCache { block_hash: *block_hash, block_number: block_header.number }
		},
	);

	let (account_peer_mapping_change_sender, _account_peer_mapping_change_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (epoch_start_sender, epoch_start_receiver) = async_broadcast::broadcast(10);

	let (cfe_settings_update_sender, _) = watch::channel::<CfeSettings>(CfeSettings::default());

	let (eth_monitor_command_sender, _eth_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_flip_command_sender, _eth_monitor_flip_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_usdc_command_sender, _eth_monitor_usdc_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_epoch_start_sender, _dot_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (dot_monitor_command_sender, _dot_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_monitor_signature_sender, _dot_monitor_signature_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_epoch_start_sender, _btc_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (btc_address_monitor_sender, _btc_address_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_tx_hash_monitor_sender, _btc_tx_hash_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	sc_observer::start(
		Arc::new(state_chain_client),
		sc_block_stream,
		EthBroadcaster::new_test(MockEthersRpcApi::new()),
		DotBroadcaster::new(MockDotRpcApi::new()),
		BtcBroadcaster::new(MockBtcRpcApi::new()),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		account_peer_mapping_change_sender,
		epoch_start_sender,
		EthAddressToMonitorSender {
			eth: eth_monitor_command_sender,
			flip: eth_monitor_flip_command_sender,
			usdc: eth_monitor_usdc_command_sender,
		},
		dot_epoch_start_sender,
		dot_monitor_command_sender,
		dot_monitor_signature_sender,
		btc_epoch_start_sender,
		btc_address_monitor_sender,
		btc_tx_hash_monitor_sender,
		cfe_settings_update_sender,
	)
	.await
	.unwrap_err();

	assert_eq!(
		epoch_start_receiver.collect::<Vec<_>>().await,
		vec![
			EpochStart::<Ethereum> {
				epoch_index: active_epoch,
				block_number: active_epoch_from_block_eth,
				current: false,
				participant: true,
				data: ()
			},
			EpochStart::<Ethereum> {
				epoch_index: current_epoch,
				block_number: current_epoch_from_block_eth,
				current: true,
				participant: false,
				data: ()
			}
		]
	);
}

#[tokio::test]
async fn does_not_start_witnessing_when_not_historic_or_current_authority() {
	let initial_epoch = 3;
	let initial_epoch_from_block_eth = 30;
	let initial_block_hash = H256::default();
	let account_id = AccountId::new([0; 32]);

	let mut state_chain_client = MockStateChainClient::new();

	state_chain_client.expect_account_id().return_once({
		let account_id = account_id.clone();
		|| account_id
	});

	state_chain_client.
expect_storage_map_entry::<pallet_cf_validator::HistoricalActiveEpochs<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash), eq(account_id))
		.once()
		.return_once(move |_, _| Ok(vec![]));
	state_chain_client
		.expect_storage_value::<pallet_cf_validator::CurrentEpoch<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(move |_| Ok(initial_epoch));
	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(initial_block_hash), eq(3))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: initial_epoch_from_block_eth,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(initial_block_hash), eq(3))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 80 }))
		});

	state_chain_client
		.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
			state_chain_runtime::Runtime,
		>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 98 }))
		});

	let sc_block_stream = tokio_stream::iter(vec![]).make_cached(
		StreamCache { block_hash: initial_block_hash, block_number: 20 },
		|(block_hash, block_header): &(state_chain_runtime::Hash, state_chain_runtime::Header)| {
			StreamCache { block_hash: *block_hash, block_number: block_header.number }
		},
	);

	let (account_peer_mapping_change_sender, _account_peer_mapping_change_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (epoch_start_sender, epoch_start_receiver) = async_broadcast::broadcast(10);
	let (cfe_settings_update_sender, _) = watch::channel::<CfeSettings>(CfeSettings::default());

	let (eth_monitor_command_sender, _eth_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_flip_command_sender, _eth_monitor_flip_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_usdc_command_sender, _eth_monitor_usdc_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_epoch_start_sender, _dot_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (dot_monitor_command_sender, _dot_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_monitor_signature_sender, _dot_monitor_signature_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_epoch_start_sender, _btc_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (btc_address_monitor_sender, _btc_address_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_tx_hash_monitor_sender, _btc_tx_hash_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	sc_observer::start(
		Arc::new(state_chain_client),
		sc_block_stream,
		EthBroadcaster::new_test(MockEthersRpcApi::new()),
		DotBroadcaster::new(MockDotRpcApi::new()),
		BtcBroadcaster::new(MockBtcRpcApi::new()),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		account_peer_mapping_change_sender,
		epoch_start_sender,
		EthAddressToMonitorSender {
			eth: eth_monitor_command_sender,
			flip: eth_monitor_flip_command_sender,
			usdc: eth_monitor_usdc_command_sender,
		},
		dot_epoch_start_sender,
		dot_monitor_command_sender,
		dot_monitor_signature_sender,
		btc_epoch_start_sender,
		btc_address_monitor_sender,
		btc_tx_hash_monitor_sender,
		cfe_settings_update_sender,
	)
	.await
	.unwrap_err();

	assert_eq!(
		epoch_start_receiver.collect::<Vec<_>>().await,
		vec![EpochStart::<Ethereum> {
			epoch_index: initial_epoch,
			block_number: initial_epoch_from_block_eth,
			current: true,
			participant: false,
			data: (),
		}]
	);
}

#[tokio::test]
async fn current_authority_to_current_authority_on_new_epoch_event() {
	let initial_epoch = 4;
	let initial_epoch_from_block_eth = 40;

	let initial_epoch_from_block_dot = 72;

	let initial_epoch_from_block_btc = 98;

	let new_epoch = 5;
	let new_epoch_from_block = 50;
	let initial_block_hash = H256::default();
	let account_id = AccountId::new([0; 32]);

	let mut state_chain_client = MockStateChainClient::new();

	state_chain_client.expect_account_id().return_once({
		let account_id = account_id.clone();
		|| account_id
	});

	state_chain_client.
expect_storage_map_entry::<pallet_cf_validator::HistoricalActiveEpochs<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash), eq(account_id.clone()))
		.once()
		.return_once(move |_, _| Ok(vec![initial_epoch]));
	state_chain_client
		.expect_storage_value::<pallet_cf_validator::CurrentEpoch<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(move |_| Ok(initial_epoch));
	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: initial_epoch_from_block_eth,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: initial_epoch_from_block_dot,
			}))
		});

	state_chain_client
		.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
			state_chain_runtime::Runtime,
		>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: initial_epoch_from_block_btc
})) 		});

	let empty_block_header = test_header(20);
	let new_epoch_block_header = test_header(21);
	let new_epoch_block_header_hash = new_epoch_block_header.hash();
	let sc_block_stream =
		tokio_stream::iter(vec![empty_block_header.clone(), new_epoch_block_header.clone()])
			.map(|block_header| (block_header.hash(), block_header))
			.make_cached(
				StreamCache { block_hash: initial_block_hash, block_number: 19 },
				|(block_hash, block_header): &(
					state_chain_runtime::Hash,
					state_chain_runtime::Header,
				)| StreamCache { block_hash: *block_hash, block_number: block_header.number },
			);
	state_chain_client
		.expect_storage_value::<frame_system::Events<state_chain_runtime::Runtime>>()
		.with(eq(empty_block_header.hash()))
		.once()
		.return_once(move |_| Ok(vec![]));
	state_chain_client
		.expect_storage_value::<frame_system::Events<state_chain_runtime::Runtime>>()
		.with(eq(new_epoch_block_header_hash))
		.once()
		.return_once(move |_| {
			Ok(vec![Box::new(frame_system::EventRecord {
				phase: Phase::ApplyExtrinsic(0),
				event: state_chain_runtime::RuntimeEvent::Validator(
					pallet_cf_validator::Event::NewEpoch(new_epoch),
				),
				topics: vec![H256::default()],
			})])
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(new_epoch_block_header_hash), eq(new_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: new_epoch_from_block,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(new_epoch_block_header_hash), eq(new_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: initial_epoch_from_block_dot,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(new_epoch_block_header_hash), eq(new_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: initial_epoch_from_block_btc
})) 		});

	state_chain_client
	.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
		state_chain_runtime::Runtime,
	>>()
	.with(eq(new_epoch_block_header_hash))
	.once()
	.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	state_chain_client.
expect_storage_double_map_entry::<pallet_cf_validator::AuthorityIndex<state_chain_runtime::Runtime>>()
		.with(eq(new_epoch_block_header_hash), eq(5), eq(account_id.clone()))
		.once()
		.return_once(move |_, _, _| Ok(Some(1)));

	let (account_peer_mapping_change_sender, _account_peer_mapping_change_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (epoch_start_sender, epoch_start_receiver) = async_broadcast::broadcast(10);

	let (cfe_settings_update_sender, _) = watch::channel::<CfeSettings>(CfeSettings::default());

	let (eth_monitor_command_sender, _eth_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_flip_command_sender, _eth_monitor_flip_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_usdc_command_sender, _eth_monitor_usdc_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_epoch_start_sender, _dot_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (dot_monitor_command_sender, _dot_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_monitor_signature_sender, _dot_monitor_signature_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_epoch_start_sender, _btc_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (btc_address_monitor_sender, _btc_address_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_tx_hash_monitor_sender, _btc_tx_hash_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	sc_observer::start(
		Arc::new(state_chain_client),
		sc_block_stream,
		EthBroadcaster::new_test(MockEthersRpcApi::new()),
		DotBroadcaster::new(MockDotRpcApi::new()),
		BtcBroadcaster::new(MockBtcRpcApi::new()),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		account_peer_mapping_change_sender,
		epoch_start_sender,
		EthAddressToMonitorSender {
			eth: eth_monitor_command_sender,
			flip: eth_monitor_flip_command_sender,
			usdc: eth_monitor_usdc_command_sender,
		},
		dot_epoch_start_sender,
		dot_monitor_command_sender,
		dot_monitor_signature_sender,
		btc_epoch_start_sender,
		btc_address_monitor_sender,
		btc_tx_hash_monitor_sender,
		cfe_settings_update_sender,
	)
	.await
	.unwrap_err();

	assert_eq!(
		epoch_start_receiver.collect::<Vec<_>>().await,
		vec![
			EpochStart::<Ethereum> {
				epoch_index: initial_epoch,
				block_number: initial_epoch_from_block_eth,
				current: true,
				participant: true,
				data: ()
			},
			EpochStart::<Ethereum> {
				epoch_index: new_epoch,
				block_number: new_epoch_from_block,
				current: true,
				participant: true,
				data: ()
			}
		]
	);
}

#[tokio::test]
async fn not_historical_to_authority_on_new_epoch() {
	let initial_epoch = 3;
	let initial_epoch_from_block_eth = 30;
	let new_epoch = 4;
	let new_epoch_from_block = 40;
	let initial_block_hash = H256::default();
	let account_id = AccountId::new([0; 32]);

	let mut state_chain_client = MockStateChainClient::new();

	state_chain_client.expect_account_id().once().return_once({
		let account_id = account_id.clone();
		|| account_id
	});

	state_chain_client.
expect_storage_map_entry::<pallet_cf_validator::HistoricalActiveEpochs<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash), eq(account_id.clone()))
		.once()
		.return_once(move |_, _| Ok(vec![]));
	state_chain_client
		.expect_storage_value::<pallet_cf_validator::CurrentEpoch<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(move |_| Ok(initial_epoch));
	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: initial_epoch_from_block_eth,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 20 }))
		});

	state_chain_client
		.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
			state_chain_runtime::Runtime,
		>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 89 }))
		});

	let empty_block_header = test_header(20);
	let new_epoch_block_header = test_header(21);
	let new_epoch_block_header_hash = new_epoch_block_header.hash();
	let sc_block_stream =
		tokio_stream::iter(vec![empty_block_header.clone(), new_epoch_block_header.clone()])
			.map(|block_header| (block_header.hash(), block_header))
			.make_cached(
				StreamCache { block_hash: initial_block_hash, block_number: 19 },
				|(block_hash, block_header): &(
					state_chain_runtime::Hash,
					state_chain_runtime::Header,
				)| StreamCache { block_hash: *block_hash, block_number: block_header.number },
			);
	state_chain_client
		.expect_storage_value::<frame_system::Events<state_chain_runtime::Runtime>>()
		.with(eq(empty_block_header.hash()))
		.once()
		.return_once(move |_| Ok(vec![]));
	state_chain_client
		.expect_storage_value::<frame_system::Events<state_chain_runtime::Runtime>>()
		.with(eq(new_epoch_block_header_hash))
		.once()
		.return_once(move |_| {
			Ok(vec![Box::new(frame_system::EventRecord {
				phase: Phase::ApplyExtrinsic(0),
				event: state_chain_runtime::RuntimeEvent::Validator(
					pallet_cf_validator::Event::NewEpoch(new_epoch),
				),
				topics: vec![H256::default()],
			})])
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(new_epoch_block_header_hash), eq(new_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: new_epoch_from_block,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(new_epoch_block_header_hash), eq(new_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 80 }))
		});

	state_chain_client
		.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
			state_chain_runtime::Runtime,
		>>()
		.with(eq(new_epoch_block_header_hash))
		.once()
		.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(new_epoch_block_header_hash), eq(new_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 120 }))
		});

	state_chain_client.
expect_storage_double_map_entry::<pallet_cf_validator::AuthorityIndex<state_chain_runtime::Runtime>>()
		.with(eq(new_epoch_block_header_hash), eq(new_epoch), eq(account_id.clone()))
		.once()
		.return_once(move |_, _, _| Ok(Some(1)));

	let (account_peer_mapping_change_sender, _account_peer_mapping_change_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (epoch_start_sender, epoch_start_receiver) = async_broadcast::broadcast(10);

	let (cfe_settings_update_sender, _) = watch::channel::<CfeSettings>(CfeSettings::default());

	let (eth_monitor_command_sender, _eth_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_flip_command_sender, _eth_monitor_flip_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_usdc_command_sender, _eth_monitor_usdc_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_epoch_start_sender, _dot_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (dot_monitor_command_sender, _dot_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_monitor_signature_sender, _dot_monitor_signature_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_epoch_start_sender, _btc_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (btc_address_monitor_sender, _btc_address_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_tx_hash_monitor_sender, _btc_tx_hash_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	sc_observer::start(
		Arc::new(state_chain_client),
		sc_block_stream,
		EthBroadcaster::new_test(MockEthersRpcApi::new()),
		DotBroadcaster::new(MockDotRpcApi::new()),
		BtcBroadcaster::new(MockBtcRpcApi::new()),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		account_peer_mapping_change_sender,
		epoch_start_sender,
		EthAddressToMonitorSender {
			eth: eth_monitor_command_sender,
			flip: eth_monitor_flip_command_sender,
			usdc: eth_monitor_usdc_command_sender,
		},
		dot_epoch_start_sender,
		dot_monitor_command_sender,
		dot_monitor_signature_sender,
		btc_epoch_start_sender,
		btc_address_monitor_sender,
		btc_tx_hash_monitor_sender,
		cfe_settings_update_sender,
	)
	.await
	.unwrap_err();

	assert_eq!(
		epoch_start_receiver.collect::<Vec<_>>().await,
		vec![
			EpochStart::<Ethereum> {
				epoch_index: initial_epoch,
				block_number: initial_epoch_from_block_eth,
				current: true,
				participant: false,
				data: ()
			},
			EpochStart::<Ethereum> {
				epoch_index: new_epoch,
				block_number: new_epoch_from_block,
				current: true,
				participant: true,
				data: ()
			}
		]
	);
}

#[tokio::test]
async fn current_authority_to_historical_on_new_epoch_event() {
	let initial_epoch = 3;
	let initial_epoch_from_block_eth = 30;
	let new_epoch = 4;
	let new_epoch_from_block = 40;
	let initial_block_hash = H256::default();
	let account_id = AccountId::new([0; 32]);

	let mut state_chain_client = MockStateChainClient::new();

	state_chain_client.expect_account_id().once().return_once({
		let account_id = account_id.clone();
		|| account_id
	});

	state_chain_client.
expect_storage_map_entry::<pallet_cf_validator::HistoricalActiveEpochs<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash), eq(account_id.clone()))
		.once()
		.return_once(move |_, _| Ok(vec![initial_epoch]));
	state_chain_client
		.expect_storage_value::<pallet_cf_validator::CurrentEpoch<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(move |_| Ok(initial_epoch));
	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(initial_block_hash), eq(3))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: initial_epoch_from_block_eth,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 20 }))
		});

	state_chain_client
		.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
			state_chain_runtime::Runtime,
		>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 120 }))
		});

	let empty_block_header = test_header(20);
	let new_epoch_block_header = test_header(21);
	let new_epoch_block_header_hash = new_epoch_block_header.hash();
	let sc_block_stream =
		tokio_stream::iter([empty_block_header.clone(), new_epoch_block_header.clone()])
			.map(|block_header| (block_header.hash(), block_header))
			.make_cached(
				StreamCache { block_hash: initial_block_hash, block_number: 19 },
				|(block_hash, block_header): &(
					state_chain_runtime::Hash,
					state_chain_runtime::Header,
				)| StreamCache { block_hash: *block_hash, block_number: block_header.number },
			);

	state_chain_client
		.expect_storage_value::<frame_system::Events<state_chain_runtime::Runtime>>()
		.with(eq(empty_block_header.hash()))
		.once()
		.return_once(move |_| Ok(vec![]));
	state_chain_client
		.expect_storage_value::<frame_system::Events<state_chain_runtime::Runtime>>()
		.with(eq(new_epoch_block_header_hash))
		.once()
		.return_once(move |_| {
			Ok(vec![Box::new(frame_system::EventRecord {
				phase: Phase::ApplyExtrinsic(0),
				event: state_chain_runtime::RuntimeEvent::Validator(
					pallet_cf_validator::Event::NewEpoch(new_epoch),
				),
				topics: vec![H256::default()],
			})])
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(new_epoch_block_header_hash), eq(new_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: new_epoch_from_block,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(new_epoch_block_header_hash), eq(new_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 80 }))
		});

	state_chain_client
			.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
				state_chain_runtime::Runtime,
			>>()
			.with(eq(new_epoch_block_header_hash))
			.once()
			.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	state_chain_client
			.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
				state_chain_runtime::Runtime,
				state_chain_runtime::BitcoinInstance,
			>>()
			.with(eq(new_epoch_block_header_hash), eq(new_epoch))
			.once()
			.return_once(move |_, _| {
				Ok(Some(Vault { public_key: Default::default(), active_from_block: 120 }))
			});

	state_chain_client.
expect_storage_double_map_entry::<pallet_cf_validator::AuthorityIndex<state_chain_runtime::Runtime>>()
		.with(eq(new_epoch_block_header_hash), eq(4), eq(account_id.clone()))
		.once()
		.return_once(move |_, _, _| Ok(None));

	let (account_peer_mapping_change_sender, _account_peer_mapping_change_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (epoch_start_sender, epoch_start_receiver) = async_broadcast::broadcast(10);

	let (cfe_settings_update_sender, _) = watch::channel::<CfeSettings>(CfeSettings::default());

	let (eth_monitor_command_sender, _eth_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_flip_command_sender, _eth_monitor_flip_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_usdc_command_sender, _eth_monitor_usdc_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_epoch_start_sender, _dot_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (dot_monitor_command_sender, _dot_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_monitor_signature_sender, _dot_monitor_signature_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_address_monitor_sender, _btc_address_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_tx_hash_monitor_sender, _btc_tx_hash_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_epoch_start_sender, _btc_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	sc_observer::start(
		Arc::new(state_chain_client),
		sc_block_stream,
		EthBroadcaster::new_test(MockEthersRpcApi::new()),
		DotBroadcaster::new(MockDotRpcApi::new()),
		BtcBroadcaster::new(MockBtcRpcApi::new()),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		account_peer_mapping_change_sender,
		epoch_start_sender,
		EthAddressToMonitorSender {
			eth: eth_monitor_command_sender,
			flip: eth_monitor_flip_command_sender,
			usdc: eth_monitor_usdc_command_sender,
		},
		dot_epoch_start_sender,
		dot_monitor_command_sender,
		dot_monitor_signature_sender,
		btc_epoch_start_sender,
		btc_address_monitor_sender,
		btc_tx_hash_monitor_sender,
		cfe_settings_update_sender,
	)
	.await
	.unwrap_err();

	assert_eq!(
		epoch_start_receiver.collect::<Vec<_>>().await,
		vec![
			EpochStart::<Ethereum> {
				epoch_index: initial_epoch,
				block_number: initial_epoch_from_block_eth,
				current: true,
				participant: true,
				data: ()
			},
			EpochStart::<Ethereum> {
				epoch_index: new_epoch,
				block_number: new_epoch_from_block,
				current: true,
				participant: false,
				data: ()
			}
		]
	);
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

	let initial_epoch = 3;
	let initial_epoch_from_block_eth = 30;

	state_chain_client.
expect_storage_map_entry::<pallet_cf_validator::HistoricalActiveEpochs<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash), eq(account_id.clone()))
		.once()
		.return_once(move |_, _| Ok(vec![initial_epoch]));
	state_chain_client
		.expect_storage_value::<pallet_cf_validator::CurrentEpoch<state_chain_runtime::Runtime>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(move |_| Ok(initial_epoch));
	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::EthereumInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault {
				public_key: Default::default(),
				active_from_block: initial_epoch_from_block_eth,
			}))
		});

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::PolkadotInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 80 }))
		});

	state_chain_client
		.expect_storage_value::<pallet_cf_environment::PolkadotVaultAccountId<
			state_chain_runtime::Runtime,
		>>()
		.with(eq(initial_block_hash))
		.once()
		.return_once(|_| Ok(Some(PolkadotAccountId::from_aliased([3u8; 32]))));

	state_chain_client
		.expect_storage_map_entry::<pallet_cf_vaults::Vaults<
			state_chain_runtime::Runtime,
			state_chain_runtime::BitcoinInstance,
		>>()
		.with(eq(initial_block_hash), eq(initial_epoch))
		.once()
		.return_once(move |_, _| {
			Ok(Some(Vault { public_key: Default::default(), active_from_block: 98 }))
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
		.expect_storage_value::<frame_system::Events<state_chain_runtime::Runtime>>()
		.with(eq(block_header.hash()))
		.once()
		.return_once(move |_| {
			Ok(vec![
				Box::new(frame_system::EventRecord {
					phase: Phase::ApplyExtrinsic(0),
					event: state_chain_runtime::RuntimeEvent::EthereumBroadcaster(
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
					event: state_chain_runtime::RuntimeEvent::EthereumBroadcaster(
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

	let (account_peer_mapping_change_sender, _account_peer_mapping_change_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (epoch_start_sender, _epoch_start_receiver) = async_broadcast::broadcast(10);

	let (cfe_settings_update_sender, _) = watch::channel::<CfeSettings>(CfeSettings::default());

	let (eth_monitor_command_sender, _eth_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_flip_command_sender, _eth_monitor_flip_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (eth_monitor_usdc_command_sender, _eth_monitor_usdc_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_epoch_start_sender, _dot_epoch_start_receiver_1) = async_broadcast::broadcast(10);

	let (dot_monitor_command_sender, _dot_monitor_command_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (dot_monitor_signature_sender, _dot_monitor_signature_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_address_monitor_sender, _btc_address_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_tx_hash_monitor_sender, _btc_tx_hash_monitor_receiver) =
		tokio::sync::mpsc::unbounded_channel();

	let (btc_epoch_start_sender, _btc_epoch_start_receiver_1) = async_broadcast::broadcast(10);

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

	sc_observer::start(
		Arc::new(state_chain_client),
		sc_block_stream,
		EthBroadcaster::new_test(ethers_rpc_mock),
		DotBroadcaster::new(MockDotRpcApi::new()),
		BtcBroadcaster::new(MockBtcRpcApi::new()),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		MockMultisigClientApi::new(),
		account_peer_mapping_change_sender,
		epoch_start_sender,
		EthAddressToMonitorSender {
			eth: eth_monitor_command_sender,
			flip: eth_monitor_flip_command_sender,
			usdc: eth_monitor_usdc_command_sender,
		},
		dot_epoch_start_sender,
		dot_monitor_command_sender,
		dot_monitor_signature_sender,
		btc_epoch_start_sender,
		btc_address_monitor_sender,
		btc_tx_hash_monitor_sender,
		cfe_settings_update_sender,
	)
	.await
	.unwrap_err();
}

// TODO: Test that when we return None for polkadot vault
// witnessing isn't started for dot, but is started for ETH

async fn should_handle_signing_request<C, I>()
where
	C: CryptoScheme + Send + Sync,
	I: 'static + Send + Sync,
	state_chain_runtime::Runtime: pallet_cf_threshold_signature::Config<I>,
	state_chain_runtime::RuntimeCall:
		std::convert::From<pallet_cf_threshold_signature::Call<state_chain_runtime::Runtime, I>>,
	<<state_chain_runtime::Runtime as pallet_cf_threshold_signature::Config<I>>::TargetChain as
ChainCrypto>::ThresholdSignature: std::convert::From<<C as CryptoScheme>::Signature>,
	Vec<C::Signature>: SignatureToThresholdSignature<
		<state_chain_runtime::Runtime as pallet_cf_threshold_signature::Config<I>>::TargetChain
	>,
{
	let first_ceremony_id = 1;
	let key_id = KeyId::new(1, [0u8; 32]);
	let payload = C::signing_payload_for_test();
	let our_account_id = AccountId32::new([0; 32]);
	let not_our_account_id = AccountId32::new([1u8; 32]);
	assert_ne!(our_account_id, not_our_account_id);

	let mut state_chain_client = MockStateChainClient::new();
	state_chain_client
		.expect_account_id()
		.times(2)
		.return_const(our_account_id.clone());
	state_chain_client.
		expect_submit_signed_extrinsic::<pallet_cf_threshold_signature::Call<state_chain_runtime::Runtime, I>>()
		.once()
		.return_once(|_| (H256::default(), extrinsic_api::signed::MockUntilFinalized::new()));
	let state_chain_client = Arc::new(state_chain_client);

	let mut multisig_client = MockMultisigClientApi::<C>::new();
	multisig_client
		.expect_update_latest_ceremony_id()
		.with(predicate::eq(first_ceremony_id))
		.once()
		.returning(|_| ());

	let next_ceremony_id = first_ceremony_id + 1;
	multisig_client
		.expect_initiate_signing()
		.with(
			predicate::eq(next_ceremony_id),
			predicate::eq(BTreeSet::from_iter([our_account_id.clone()])),
			predicate::eq(vec![(key_id.clone(), payload.clone())]),
		)
		.once()
		.return_once(|_, _, _| {
			futures::future::ready(Err((
				BTreeSet::new(),
				SigningFailureReason::InvalidParticipants,
			)))
			.boxed()
		});

	task_scope(|scope| {
		async {
			// Handle a signing request that we are not participating in
			sc_observer::handle_signing_request::<_, _, C, I>(
				scope,
				&multisig_client,
				state_chain_client.clone(),
				first_ceremony_id,
				BTreeSet::from_iter([not_our_account_id.clone()]),
				vec![(key_id.clone(), payload.clone())],
			)
			.await;

			// Handle a signing request that we are participating in
			sc_observer::handle_signing_request::<_, _, C, I>(
				scope,
				&multisig_client,
				state_chain_client.clone(),
				next_ceremony_id,
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
	should_handle_signing_request::<EthSigning, EthereumInstance>().await;
}

mod dot_signing {

	use multisig::polkadot::PolkadotSigning;

	use super::*;
	use state_chain_runtime::PolkadotInstance;

	#[tokio::test]
	async fn should_handle_signing_request_dot() {
		should_handle_signing_request::<PolkadotSigning, PolkadotInstance>().await;
	}
}

async fn should_handle_keygen_request<C, I>()
where
	C: CryptoScheme<Chain = <state_chain_runtime::Runtime as pallet_cf_vaults::Config<I>>::Chain>
		+ Send
		+ Sync,
	I: CryptoCompat<C, C::Chain> + 'static + Send + Sync,
	state_chain_runtime::Runtime: pallet_cf_vaults::Config<I>,
	state_chain_runtime::RuntimeCall:
		std::convert::From<pallet_cf_vaults::Call<state_chain_runtime::Runtime, I>>,
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
		.expect_submit_signed_extrinsic::<pallet_cf_vaults::Call<state_chain_runtime::Runtime, I>>()
		.once()
		.return_once(|_| (H256::default(), extrinsic_api::signed::MockUntilFinalized::new()));
	let state_chain_client = Arc::new(state_chain_client);

	let mut multisig_client = MockMultisigClientApi::<C>::new();
	multisig_client
		.expect_update_latest_ceremony_id()
		.with(predicate::eq(first_ceremony_id))
		.once()
		.return_once(|_| ());

	let next_ceremony_id = first_ceremony_id + 1;
	// Set up the mock api to expect the keygen and sign calls for the ceremonies we are
	// participating in. It doesn't matter what failure reasons they return.
	multisig_client
		.expect_initiate_keygen()
		.with(
			predicate::eq(next_ceremony_id),
			predicate::eq(GENESIS_EPOCH),
			predicate::eq(BTreeSet::from_iter([our_account_id.clone()])),
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
	use state_chain_runtime::PolkadotInstance;
	#[tokio::test]
	async fn should_handle_keygen_request_dot() {
		should_handle_keygen_request::<PolkadotSigning, PolkadotInstance>().await;
	}
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

			let (epoch_start_sender, _epoch_start_receiver) = async_broadcast::broadcast(10);

			let (cfe_settings_update_sender, _) =
				watch::channel::<CfeSettings>(CfeSettings::default());

			let (eth_monitor_command_sender, _eth_monitor_command_receiver) =
				tokio::sync::mpsc::unbounded_channel();

			let (eth_monitor_flip_command_sender, _eth_monitor_flip_command_receiver) =
				tokio::sync::mpsc::unbounded_channel();

			let (eth_monitor_usdc_command_sender, _eth_monitor_usdc_command_receiver) =
				tokio::sync::mpsc::unbounded_channel();

			let (dot_epoch_start_sender, _dot_epoch_start_receiver_1) =
				async_broadcast::broadcast(10);

			let (dot_monitor_command_sender, _dot_monitor_command_receiver) =
				tokio::sync::mpsc::unbounded_channel();

			let (dot_monitor_signature_sender, _dot_monitor_signature_receiver) =
				tokio::sync::mpsc::unbounded_channel();

			let (btc_epoch_start_sender, _btc_epoch_start_receiver_1) =
				async_broadcast::broadcast(10);

			let (btc_address_monitor_sender, _btc_address_monitor_receiver) =
				tokio::sync::mpsc::unbounded_channel();

			let (btc_tx_hash_monitor_sender, _btc_tx_hash_monitor_receiver) =
				tokio::sync::mpsc::unbounded_channel();

			sc_observer::start(
				state_chain_client,
				sc_block_stream,
				EthBroadcaster::new(EthersRpcClient::new(&settings.eth).await.unwrap()),
				DotBroadcaster::new(MockDotRpcApi::new()),
				BtcBroadcaster::new(MockBtcRpcApi::new()),
				MockMultisigClientApi::new(),
				MockMultisigClientApi::new(),
				MockMultisigClientApi::new(),
				account_peer_mapping_change_sender,
				epoch_start_sender,
				EthAddressToMonitorSender {
					eth: eth_monitor_command_sender,
					flip: eth_monitor_flip_command_sender,
					usdc: eth_monitor_usdc_command_sender,
				},
				dot_epoch_start_sender,
				dot_monitor_command_sender,
				dot_monitor_signature_sender,
				btc_epoch_start_sender,
				btc_address_monitor_sender,
				btc_tx_hash_monitor_sender,
				cfe_settings_update_sender,
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

#[tokio::test]
async fn test_ensure_reported_parties_are_participants() {
	use sc_observer::ensure_reported_parties_are_participants;

	fn to_account_id_set<T: AsRef<[u8]>>(ids: T) -> BTreeSet<AccountId> {
		ids.as_ref().iter().map(|i| AccountId::new([*i; 32])).collect()
	}

	// Test a few different common scenarios that should be fine
	ensure_reported_parties_are_participants(
		&to_account_id_set(vec![0, 2]),
		&to_account_id_set(vec![0, 1, 2, 3]),
	);
	ensure_reported_parties_are_participants(
		&BTreeSet::new(),
		&to_account_id_set(vec![0, 1, 2, 3]),
	);

	// Test the panic case, one of the reported parties is not a participant.
	assert_future_panics!(async move {
		ensure_reported_parties_are_participants(
			&to_account_id_set(vec![0, 1, 2, 3]),
			&to_account_id_set(vec![0, 1, 2]),
		);
	});
}
