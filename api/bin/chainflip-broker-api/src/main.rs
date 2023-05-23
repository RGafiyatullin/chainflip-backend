use anyhow::anyhow;
use chainflip_api::{
	self, clean_foreign_chain_address,
	primitives::{AccountRole, Asset, BasisPoints, CcmDepositMetadata},
	settings::StateChain,
};
use clap::Parser;
use jsonrpsee::{
	core::{async_trait, Error},
	proc_macros::rpc,
	server::ServerBuilder,
};
use serde_json::json;
use std::path::PathBuf;

#[rpc(server, client, namespace = "broker")]
pub trait Rpc {
	#[method(name = "registerAccount")]
	async fn register_account(&self) -> Result<String, Error>;

	#[method(name = "requestSwapDepositAddress")]
	async fn request_swap_deposit_address(
		&self,
		source_asset: Asset,
		destination_asset: Asset,
		destination_address: String,
		broker_commission_bps: BasisPoints,
		message_metadata: Option<CcmDepositMetadata>,
	) -> Result<String, Error>;
}

pub struct RpcServerImpl {
	state_chain_settings: StateChain,
}

impl RpcServerImpl {
	pub fn new(BrokerOptions { ws_endpoint, signing_key_file, .. }: BrokerOptions) -> Self {
		Self { state_chain_settings: StateChain { ws_endpoint, signing_key_file } }
	}
}

#[async_trait]
impl RpcServer for RpcServerImpl {
	async fn register_account(&self) -> Result<String, Error> {
		Ok(chainflip_api::register_account_role(AccountRole::Broker, &self.state_chain_settings)
			.await
			.map(|tx_hash| format!("{tx_hash:#x}"))?)
	}
	async fn request_swap_deposit_address(
		&self,
		source_asset: Asset,
		destination_asset: Asset,
		destination_address: String,
		broker_commission_bps: BasisPoints,
		message_metadata: Option<CcmDepositMetadata>,
	) -> Result<String, Error> {
		Ok(chainflip_api::request_swap_deposit_address(
			&self.state_chain_settings,
			source_asset,
			destination_asset,
			clean_foreign_chain_address(destination_asset.into(), &destination_address)?,
			broker_commission_bps,
			message_metadata,
		)
		.await
		.map(|(address, expiry_block)| {
			json!({ "address": address.to_string(), "expiry_block": expiry_block }).to_string()
		})
		.map_err(|e| anyhow!("{}:{}", e, e.root_cause()))?)
	}
}

#[derive(Parser, Debug, Clone, Default)]
pub struct BrokerOptions {
	#[clap(
		long = "port",
		default_value = "80",
		help = "The port number on which the broker will listen for connections. Use 0 to assign a random port."
	)]
	pub port: u16,
	#[clap(
		long = "state_chain.ws_endpoint",
		default_value = "ws://localhost:9944",
		help = "The state chain node's RPC endpoint."
	)]
	pub ws_endpoint: String,
	#[clap(
		long = "state_chain.signing_key_file",
		default_value = "/etc/chainflip/keys/signing_key_file",
		help = "A path to a file that contains the broker's secret key for signing extrinsics."
	)]
	pub signing_key_file: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let opts = BrokerOptions::parse();
	chainflip_api::use_chainflip_account_id_encoding();
	tracing_subscriber::FmtSubscriber::builder()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init()
		.expect("setting default subscriber failed");

	let server = ServerBuilder::default().build(format!("0.0.0.0:{}", opts.port)).await?;
	let server_addr = server.local_addr()?;
	let server = server.start(RpcServerImpl::new(opts).into_rpc())?;

	println!("🎙 Server is listening on {server_addr}.");

	server.stopped().await;

	Ok(())
}
