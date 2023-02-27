use anyhow::Result;
use async_trait::async_trait;
use frame_support::dispatch::DispatchInfo;
use sp_core::H256;
use sp_runtime::DispatchError;
use state_chain_runtime::AccountId;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

#[derive(Error, Debug)]
pub enum ExtrinsicFinalizationError {
	#[error("The requested transaction was included in a finalized block with tx_hash: {0:#?}")]
	Finalized(H256, DispatchInfo, Vec<state_chain_runtime::RuntimeEvent>, DispatchError),
	#[error("The requested transaction was not and will not be included in a finalized block")]
	NotFinalized,
	#[error(
		"The requested transaction was not (but maybe in the future) included in a finalized block"
	)]
	Unknown,
}

pub type ExtrinsicFinalizationResult =
	Result<(H256, DispatchInfo, Vec<state_chain_runtime::RuntimeEvent>), ExtrinsicFinalizationError>;

// Note 'static on the generics in this trait are only required for mockall to mock it
#[async_trait]
pub trait SignedExtrinsicApi {
	fn account_id(&self) -> AccountId;

	async fn request_signed_extrinsic<Call>(
		&self,
		call: Call,
		logger: &slog::Logger,
	) -> ExtrinsicFinalizationResult
	where
		Call: Into<state_chain_runtime::RuntimeCall>
			+ Clone
			+ std::fmt::Debug
			+ Send
			+ Sync
			+ 'static;
}

// Note 'static on the generics in this trait are only required for mockall to mock it
#[async_trait]
pub trait UnsignedExtrinsicApi {
	async fn submit_unsigned_extrinsic<Call>(
		&self,
		call: Call,
		logger: &slog::Logger,
	) -> Result<H256>
	where
		Call: Into<state_chain_runtime::RuntimeCall>
			+ Clone
			+ std::fmt::Debug
			+ Send
			+ Sync
			+ 'static;
}

impl<BaseRpcApi: super::base_rpc_api::BaseRpcApi + Send + Sync + 'static>
	super::StateChainClient<BaseRpcApi>
{
	async fn send_request_and_receive_result<Call, Result: Send>(
		request_sender: &mpsc::UnboundedSender<(
			state_chain_runtime::RuntimeCall,
			oneshot::Sender<Result>,
		)>,
		call: Call,
	) -> Result
	where
		Call: Into<state_chain_runtime::RuntimeCall> + Clone + std::fmt::Debug,
	{
		let (extrinsic_result_sender, extrinsic_result_receiver) = oneshot::channel();

		{
			let _result = request_sender.send((call.clone().into(), extrinsic_result_sender));
		}

		extrinsic_result_receiver.await.expect("Backend failed") // TODO: This type of error in the codebase is currently handled inconsistently
	}
}

#[async_trait]
impl<BaseRpcApi: super::base_rpc_api::BaseRpcApi + Send + Sync + 'static> SignedExtrinsicApi
	for super::StateChainClient<BaseRpcApi>
{
	fn account_id(&self) -> AccountId {
		self.account_id.clone()
	}

	/// Sign, submit, and watch an extrinsic retrying if submissions fail be to finalized
	async fn finalize_signed_extrinsic<Call>(
		&self,
		call: Call,
		_logger: &slog::Logger,
	) -> ExtrinsicFinalizationResult
	where
		Call: Into<state_chain_runtime::RuntimeCall>
			+ Clone
			+ std::fmt::Debug
			+ Send
			+ Sync
			+ 'static,
	{
		Self::send_request_and_receive_result(&self.signed_extrinsic_request_sender, call).await
	}
}

#[async_trait]
impl<BaseRpcApi: super::base_rpc_api::BaseRpcApi + Send + Sync + 'static> UnsignedExtrinsicApi
	for super::StateChainClient<BaseRpcApi>
{
	/// Submit an unsigned extrinsic.
	async fn submit_unsigned_extrinsic<Call>(
		&self,
		call: Call,
		_logger: &slog::Logger,
	) -> Result<H256>
	where
		Call: Into<state_chain_runtime::RuntimeCall>
			+ std::fmt::Debug
			+ Clone
			+ Send
			+ Sync
			+ 'static,
	{
		Self::send_request_and_receive_result(&self.unsigned_extrinsic_request_sender, call).await
	}
}
