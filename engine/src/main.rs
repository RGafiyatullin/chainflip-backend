use std::sync::{atomic::AtomicBool, Arc};

use anyhow::Context;

use cf_primitives::AccountRole;
use chainflip_engine::{
	btc::{self, rpc::BtcRpcClient, BtcBroadcaster},
	db::{KeyStore, PersistentKeyDB},
	dot::{rpc::LoggingRpcClient, DotBroadcaster},
	eth::{self, broadcaster::EthBroadcaster, build_broadcast_channel},
	health, p2p,
	settings::{CommandLineOptions, Settings},
	state_chain_observer::{
		self,
		client::{
			extrinsic_api::signed::{SignedExtrinsicApi, UntilFinalized},
			storage_api::StorageApi,
		},
	},
};
use multisig::{self, bitcoin::BtcSigning, eth::EthSigning, polkadot::PolkadotSigning};
use tracing::log;
use utilities::task_scope::task_scope;

use crate::eth::ethers_rpc::EthersRpcClient;
use chainflip_node::chain_spec::use_chainflip_account_id_encoding;
use clap::Parser;
use futures::FutureExt;
use pallet_cf_validator::SemVer;
use utilities::CachedStream;
use web3::types::U256;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	use_chainflip_account_id_encoding();

	let settings = Settings::new(CommandLineOptions::parse()).context("Error reading settings")?;

	// Note: the greeting should only be printed in normal mode (i.e. not for short-lived commands
	// like `--version`), so we execute it only after the settings have been parsed.
	utilities::print_starting!();

	task_scope(|scope| {
		async move {
			let has_completed_initialising = Arc::new(AtomicBool::new(false));

			if let Some(health_check_settings) = &settings.health_check {
				health::start(scope, health_check_settings, has_completed_initialising.clone())
					.await?;
			}

			utilities::init_json_logger(scope).await;

			let (state_chain_stream, state_chain_client) =
				state_chain_observer::client::StateChainClient::connect_with_account(
					scope,
					&settings.state_chain.ws_endpoint,
					&settings.state_chain.signing_key_file,
					AccountRole::Validator,
					true,
				)
				.await?;

			let expected_chain_id = U256::from(
				state_chain_client
					.storage_value::<pallet_cf_environment::EthereumChainId<state_chain_runtime::Runtime>>(
						state_chain_stream.cache().block_hash,
					)
					.await
					.context("Failed to get EthereumChainId from state chain")?,
			);

			let btc_rpc_client =
				BtcRpcClient::new(&settings.btc).context("Failed to create Bitcoin Client")?;

			state_chain_client
				.finalize_signed_extrinsic(pallet_cf_validator::Call::cfe_version {
					new_version: SemVer {
						major: env!("CARGO_PKG_VERSION_MAJOR").parse::<u8>().unwrap(),
						minor: env!("CARGO_PKG_VERSION_MINOR").parse::<u8>().unwrap(),
						patch: env!("CARGO_PKG_VERSION_PATCH").parse::<u8>().unwrap(),
					},
				})
				.await
				.until_finalized()
				.await
				.context("Failed to submit version to state chain")?;

			let (epoch_start_sender, [epoch_start_receiver_1, epoch_start_receiver_2]) =
				build_broadcast_channel(10);

			let (dot_epoch_start_sender, [mut dot_epoch_start_receiver]) =
				build_broadcast_channel(10);

			let (btc_epoch_start_sender, [btc_epoch_start_receiver]) = build_broadcast_channel(10);

			let cfe_settings = state_chain_client
				.storage_value::<pallet_cf_environment::CfeSettings<state_chain_runtime::Runtime>>(
					state_chain_stream.cache().block_hash,
				)
				.await
				.context("Failed to get on chain CFE settings from SC")?;

			let (cfe_settings_update_sender, cfe_settings_update_receiver) =
				tokio::sync::watch::channel(cfe_settings);

			let db = Arc::new(
				PersistentKeyDB::open_and_migrate_to_latest(
					settings.signing.db_file.as_path(),
					Some(state_chain_client.genesis_hash()),
				)
				.context("Failed to open database")?,
			);

			let (
				eth_outgoing_sender,
				eth_incoming_receiver,
				dot_outgoing_sender,
				dot_incoming_receiver,
				btc_outgoing_sender,
				btc_incoming_receiver,
				peer_update_sender,
				p2p_fut,
			) = p2p::start(
				state_chain_client.clone(),
				settings.node_p2p,
				state_chain_stream.cache().block_hash,
			)
			.await
			.context("Failed to start p2p module")?;

			scope.spawn(p2p_fut);

			let (eth_multisig_client, eth_multisig_client_backend_future) =
				chainflip_engine::multisig::start_client::<EthSigning>(
					state_chain_client.account_id(),
					KeyStore::new(db.clone()),
					eth_incoming_receiver,
					eth_outgoing_sender,
					state_chain_client
						.storage_value::<pallet_cf_vaults::CeremonyIdCounter<
							state_chain_runtime::Runtime,
							state_chain_runtime::EthereumInstance,
						>>(state_chain_stream.cache().block_hash)
						.await
						.context("Failed to get Ethereum CeremonyIdCounter from SC")?,
				);

			scope.spawn(eth_multisig_client_backend_future);

			let (dot_multisig_client, dot_multisig_client_backend_future) =
				chainflip_engine::multisig::start_client::<PolkadotSigning>(
					state_chain_client.account_id(),
					KeyStore::new(db.clone()),
					dot_incoming_receiver,
					dot_outgoing_sender,
					state_chain_client
						.storage_value::<pallet_cf_vaults::CeremonyIdCounter<
							state_chain_runtime::Runtime,
							state_chain_runtime::PolkadotInstance,
						>>(state_chain_stream.cache().block_hash)
						.await
						.context("Failed to get Polkadot CeremonyIdCounter from SC")?,
				);

			scope.spawn(dot_multisig_client_backend_future);

			let (btc_multisig_client, btc_multisig_client_backend_future) =
				chainflip_engine::multisig::start_client::<BtcSigning>(
					state_chain_client.account_id(),
					KeyStore::new(db.clone()),
					btc_incoming_receiver,
					btc_outgoing_sender,
					state_chain_client
						.storage_value::<pallet_cf_vaults::CeremonyIdCounter<
							state_chain_runtime::Runtime,
							state_chain_runtime::BitcoinInstance,
						>>(state_chain_stream.cache().block_hash)
						.await
						.context("Failed to get Bitcoin CeremonyIdCounter from SC")?,
				);

			scope.spawn(btc_multisig_client_backend_future);

			let eth_address_to_monitor = eth::witnessing::start(
				scope,
				&settings.eth,
				state_chain_client.clone(),
				expected_chain_id,
				state_chain_stream.cache().block_hash,
				epoch_start_receiver_1,
				epoch_start_receiver_2,
				cfe_settings_update_receiver,
				db.clone(),
			)
			.await?;

			let (btc_monitor_command_sender, btc_tx_hash_sender) = btc::witnessing::start(
				scope,
				state_chain_client.clone(),
				&settings.btc,
				btc_epoch_start_receiver,
				state_chain_stream.cache().block_hash,
				db.clone(),
			)
			.await?;

			let (dot_monitor_address_sender, mut dot_monitor_address_receiver) =
				tokio::sync::mpsc::unbounded_channel();
			let (dot_monitor_signature_sender, mut dot_monitor_signature_receiver) =
				tokio::sync::mpsc::unbounded_channel();

			scope.spawn(async move {
				while let Some(a) = dot_monitor_address_receiver.recv().await {
					log::debug!("Ignoring dot monitor address {a:?}");
				}
				log::debug!("dot monitor address channel closed");
				Ok(())
			});

			scope.spawn(async move {
				while let Some(s) = dot_monitor_signature_receiver.recv().await {
					log::debug!("Ignoring dot monitor signature {s:?}");
				}
				log::debug!("dot monitor signature channel closed");
				Ok(())
			});

			scope.spawn(async move {
				while let Ok(e) = dot_epoch_start_receiver.recv().await {
					log::debug!("Ignoring dot epoch start {e:?}");
				}
				log::debug!("dot epoch start channel closed");
				Ok(())
			});

			scope.spawn(state_chain_observer::start(
				state_chain_client.clone(),
				state_chain_stream.clone(),
				EthBroadcaster::new(EthersRpcClient::new(&settings.eth).await?),
				DotBroadcaster::new(LoggingRpcClient),
				BtcBroadcaster::new(btc_rpc_client.clone()),
				eth_multisig_client,
				dot_multisig_client,
				btc_multisig_client,
				peer_update_sender,
				epoch_start_sender,
				eth_address_to_monitor,
				dot_epoch_start_sender,
				dot_monitor_address_sender,
				dot_monitor_signature_sender,
				btc_epoch_start_sender,
				btc_monitor_command_sender,
				btc_tx_hash_sender,
				cfe_settings_update_sender,
			));

			has_completed_initialising.store(true, std::sync::atomic::Ordering::Relaxed);

			Ok(())
		}
		.boxed()
	})
	.await
}
