pub mod base_rpc_api;
pub mod extrinsic_api;
mod signer;
pub mod storage_api;

use base_rpc_api::BaseRpcApi;

use anyhow::{anyhow, bail, Context, Result};
use cf_primitives::AccountRole;
use frame_support::{dispatch::DispatchInfo, pallet_prelude::InvalidTransaction};
use futures::{Stream, StreamExt, TryStreamExt};

use itertools::Itertools;
use slog::o;
use sp_core::{Pair, H256};
use sp_runtime::{traits::Hash, DispatchError, MultiAddress};
use state_chain_runtime::AccountId;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{mpsc, oneshot};

use crate::{
	common::{read_clean_and_decode_hex_str_file, EngineTryStreamExt},
	constants::SIGNED_EXTRINSIC_LIFETIME,
	logging::COMPONENT_KEY,
	settings,
	state_chain_observer::client::storage_api::StorageApi,
	task_scope::{Scope, ScopedJoinHandle},
};

pub struct StateChainClient<
	BaseRpcClient = base_rpc_api::BaseRpcClient<jsonrpsee::ws_client::WsClient>,
> {
	genesis_hash: state_chain_runtime::Hash,
	account_id: AccountId,
	signed_extrinsic_request_sender: mpsc::UnboundedSender<(
		state_chain_runtime::RuntimeCall,
		oneshot::Sender<Result<H256, anyhow::Error>>,
	)>,
	unsigned_extrinsic_request_sender: mpsc::UnboundedSender<(
		state_chain_runtime::RuntimeCall,
		oneshot::Sender<Result<H256, anyhow::Error>>,
	)>,
	_block_producer: ScopedJoinHandle<()>,
	_unsigned_extrinsic_consumer: ScopedJoinHandle<()>,
	_signed_extrinsic_consumer: ScopedJoinHandle<()>,
	pub base_rpc_client: Arc<BaseRpcClient>,
}

impl<BaseRpcClient> StateChainClient<BaseRpcClient> {
	pub fn get_genesis_hash(&self) -> state_chain_runtime::Hash {
		self.genesis_hash
	}
}

fn invalid_err_obj(invalid_reason: InvalidTransaction) -> jsonrpsee::types::ErrorObjectOwned {
	jsonrpsee::types::ErrorObject::owned(
		1010,
		"Invalid Transaction",
		Some(<&'static str>::from(invalid_reason)),
	)
}

/// This resolves a compiler bug: https://github.com/rust-lang/rust/issues/102211#issuecomment-1372215393
/// We should be able to remove this in future versions of the rustc
fn assert_stream_send<'u, R>(
	stream: impl 'u + Send + Stream<Item = R>,
) -> impl 'u + Send + Stream<Item = R> {
	stream
}

impl StateChainClient {
	pub async fn new<'a>(
		scope: &Scope<'a, anyhow::Error>,
		state_chain_settings: &settings::StateChain,
		required_role: AccountRole,
		wait_for_required_role: bool,
		logger: &slog::Logger,
	) -> Result<(H256, impl Stream<Item = state_chain_runtime::Header>, Arc<StateChainClient>)> {
		Self::inner_new(scope, state_chain_settings, required_role, wait_for_required_role, logger)
			.await
			.context("Failed to initialize StateChainClient")
	}

	async fn inner_new<'a>(
		scope: &Scope<'a, anyhow::Error>,
		state_chain_settings: &settings::StateChain,
		required_role: AccountRole,
		wait_for_required_role: bool,
		logger: &slog::Logger,
	) -> Result<(H256, impl Stream<Item = state_chain_runtime::Header>, Arc<StateChainClient>)> {
		let logger = logger.new(o!(COMPONENT_KEY => "StateChainClient"));
		let signer = signer::PairSigner::<sp_core::sr25519::Pair>::new(
			sp_core::sr25519::Pair::from_seed(&read_clean_and_decode_hex_str_file(
				&state_chain_settings.signing_key_file,
				"Signing Key",
				|str| {
					<[u8; 32]>::try_from(hex::decode(str).map_err(anyhow::Error::new)?)
						.map_err(|_err| anyhow!("Wrong length"))
				},
			)?),
		);

		let base_rpc_client =
			Arc::new(base_rpc_api::BaseRpcClient::new(state_chain_settings).await?);

		let genesis_hash = base_rpc_client.block_hash(0).await?.unwrap();

		let (first_finalized_block_header, mut finalized_block_header_stream) = {
			// https://substrate.stackexchange.com/questions/3667/api-rpc-chain-subscribefinalizedheads-missing-blocks
			// https://arxiv.org/abs/2007.01560
			let mut sparse_finalized_block_header_stream = base_rpc_client
				.subscribe_finalized_block_headers()
				.await?
				.map_err(Into::into)
				.chain(futures::stream::once(std::future::ready(Err(anyhow::anyhow!(
					"sparse_finalized_block_header_stream unexpectedly ended"
				)))));

			let mut latest_finalized_header: state_chain_runtime::Header =
				sparse_finalized_block_header_stream.next().await.unwrap()?;
			let base_rpc_client = base_rpc_client.clone();

			(
				latest_finalized_header.clone(),
				assert_stream_send(Box::pin(
					sparse_finalized_block_header_stream
						.and_then(move |next_finalized_header| {
							assert!(latest_finalized_header.number < next_finalized_header.number);

							let prev_finalized_header = std::mem::replace(
								&mut latest_finalized_header,
								next_finalized_header.clone(),
							);

							let base_rpc_client = base_rpc_client.clone();
							async move {
								let base_rpc_client = &base_rpc_client;
								let intervening_headers: Vec<_> = futures::stream::iter(
									prev_finalized_header.number + 1..next_finalized_header.number,
								)
								.then(|block_number| async move {
									let block_hash =
										base_rpc_client.block_hash(block_number).await?.unwrap();
									let block_header =
										base_rpc_client.block_header(block_hash).await?;
									assert_eq!(block_header.hash(), block_hash);
									assert_eq!(block_header.number, block_number);
									Result::<_, anyhow::Error>::Ok((block_hash, block_header))
								})
								.try_collect()
								.await?;

								for (block_hash, next_block_header) in Iterator::zip(
									std::iter::once(&prev_finalized_header.hash()).chain(
										intervening_headers.iter().map(|(hash, _header)| hash),
									),
									intervening_headers
										.iter()
										.map(|(_hash, header)| header)
										.chain(std::iter::once(&next_finalized_header)),
								) {
									assert_eq!(*block_hash, next_block_header.parent_hash);
								}

								Result::<_, anyhow::Error>::Ok(futures::stream::iter(
									intervening_headers
										.into_iter()
										.map(|(_hash, header)| header)
										.chain(std::iter::once(next_finalized_header))
										.map(Result::<_, anyhow::Error>::Ok),
								))
							}
						})
						.end_after_error()
						.try_flatten(),
				)),
			)
		};

		// Often `finalized_header` returns a significantly newer latest block than the stream
		// returns so we move the stream forward to this block
		let (mut latest_block_hash, mut latest_block_number) = {
			let finalised_header_hash = base_rpc_client.latest_finalized_block_hash().await?;
			let finalised_header = base_rpc_client.block_header(finalised_header_hash).await?;

			if first_finalized_block_header.number < finalised_header.number {
				for block_number in
					first_finalized_block_header.number + 1..=finalised_header.number
				{
					assert_eq!(
						finalized_block_header_stream.next().await.unwrap()?.number,
						block_number
					);
				}
				(finalised_header_hash, finalised_header.number)
			} else {
				(first_finalized_block_header.hash(), first_finalized_block_header.number)
			}
		};

		let (latest_block_hash, latest_block_number, account_nonce) = {
			loop {
				match base_rpc_client
					.storage_map_entry::<pallet_cf_account_roles::AccountRoles<state_chain_runtime::Runtime>>(
						latest_block_hash,
						&signer.account_id,
					)
					.await?
				{
					Some(role) =>
						if required_role == AccountRole::None || required_role == role {
							break
						} else if wait_for_required_role && role == AccountRole::None {
							slog::warn!(logger, "Your Chainflip account {} does not have an assigned account role. WAITING for the account role to be set to '{:?}' at block: {}", signer.account_id, required_role, latest_block_number);
						} else {
							bail!("Your Chainflip account {} has the wrong account role '{:?}'. The '{:?}' account role is required", signer.account_id, role, required_role);
						},
					None =>
						if wait_for_required_role {
							slog::warn!(logger, "Your Chainflip account {} is not staked. Note, if you have already staked, it may take some time for your stake to be detected. WAITING for your account to be staked at block: {}", signer.account_id, latest_block_number);
						} else {
							bail!("Your Chainflip account {} is not staked", signer.account_id);
						},
				}

				let block_header = finalized_block_header_stream.next().await.unwrap()?;
				latest_block_hash = block_header.hash();
				latest_block_number += 1;
				assert_eq!(latest_block_number, block_header.number);
			}

			(
				latest_block_hash,
				latest_block_number,
				base_rpc_client
					.storage_map_entry::<frame_system::Account<state_chain_runtime::Runtime>>(
						latest_block_hash,
						&signer.account_id,
					)
					.await?
					.nonce,
			)
		};

		// These are unbounded to avoid deadlock between sending blocks and receiving extrinsics
		let (signed_extrinsic_request_sender, mut signed_extrinsic_request_receiver) =
			mpsc::unbounded_channel();
		let (unsigned_extrinsic_request_sender, mut unsigned_extrinsic_request_receiver) =
			mpsc::unbounded_channel();

		const BLOCK_CAPACITY: usize = 10;
		let (block_sender, block_receiver) =
			async_broadcast::broadcast::<state_chain_runtime::Header>(BLOCK_CAPACITY);

		let state_chain_client = Arc::new(StateChainClient {
			genesis_hash,
			account_id: signer.account_id.clone(),
			signed_extrinsic_request_sender,
			unsigned_extrinsic_request_sender,
			_signed_extrinsic_consumer: scope.spawn_with_handle({
				let logger = logger.clone();
				let base_rpc_client = base_rpc_client.clone();
				let mut signed_extrinsic_consumer_block_receiver = block_receiver.clone();

				let mut runtime_version = base_rpc_client.runtime_version().await?;

				let mut finalized_nonce = account_nonce;
				let mut anticipated_nonce = account_nonce;

				let mut latest_block_number = latest_block_number;
				let mut latest_block_hash = latest_block_hash;

				async move {
					type ExtrinsicRequestID = u64;

					enum ExtrinsicFailure {
						/// The requested transaction was included in a finalized block
						Finalized(DispatchInfo, DispatchError, Vec<state_chain_runtime::RuntimeEvent>),
						TimedOut(NonFinalizedStatus),
					}

					enum NonFinalizedStatus {
						Guaranteed,
						Unknown
					}

					struct ExtrinsicRequest {
						submissions: usize,
						failed_submissions: usize,
						allow_unknown_finalized_status_on_death: bool,
						lifetime: std::ops::RangeToInclusive<cf_primitives::BlockNumber>,
						call: state_chain_runtime::RuntimeCall,
						result_sender: oneshot::Sender<Result<(DispatchInfo, Vec<state_chain_runtime::RuntimeEvent>), ExtrinsicFailure>>,
					}

					struct SignedExtrinsicSubmission {
						lifetime: std::ops::RangeTo<cf_primitives::BlockNumber>,
						tx_hash: H256,
						request_id: ExtrinsicRequestID,
					}

					struct SignedExtrinsicSubmitter {
						signer: signer::PairSigner<sp_core::sr25519::Pair>,
						anticipated_nonce: state_chain_runtime::Index,
						finalized_nonce: state_chain_runtime::Index,
						runtime_version: sp_version::RuntimeVersion,
						submissions_by_nonce: BTreeMap<state_chain_runtime::Index, Vec<SignedExtrinsicSubmission>>,
						base_rpc_client: Arc<base_rpc_api::BaseRpcClient<jsonrpsee::ws_client::WsClient>>,
					}

					enum SubmissionLogicError {
						NonceTooLow,
					}

					impl SignedExtrinsicSubmitter {
						fn new(signer: signer::PairSigner<sp_core::sr25519::Pair>, finalized_nonce: state_chain_runtime::Index, runtime_version: sp_version::RuntimeVersion, base_rpc_client: Arc<base_rpc_api::BaseRpcClient<jsonrpsee::ws_client::WsClient>>) -> Self {
							Self {
								signer,
								anticipated_nonce: finalized_nonce,
								finalized_nonce,
								runtime_version,
								submissions_by_nonce: Default::default(),
								base_rpc_client,
							}
						}

						async fn try_submit_extrinsic(&mut self, call: state_chain_runtime::RuntimeCall, nonce: state_chain_runtime::Index, genesis_hash: H256, latest_block_hash: H256, latest_block_number: state_chain_runtime::BlockNumber, request_id: ExtrinsicRequestID) -> Result<Result<(), SubmissionLogicError>, anyhow::Error> {
							loop {
								let (signed_extrinsic, lifetime) = self.signer.new_signed_extrinsic(
									call.clone(),
									&self.runtime_version,
									genesis_hash,
									latest_block_hash,
									latest_block_number,
									SIGNED_EXTRINSIC_LIFETIME,
									nonce,
								);

								match self.base_rpc_client
									.submit_extrinsic(signed_extrinsic)
									.await
								{
									Ok(tx_hash) => {
										self.submissions_by_nonce.entry(self.anticipated_nonce).or_default().push(SignedExtrinsicSubmission {
											lifetime,
											tx_hash,
											request_id,
										});
										break Ok(Ok(()))
									},
									Err(rpc_err) => match rpc_err {
										// This occurs when a transaction with the same nonce is in the transaction pool
										// (and the priority is <= priority of that existing tx)
										jsonrpsee::core::Error::Call(jsonrpsee::types::error::CallError::Custom(ref obj)) if obj.code() == 1014 => {
											break Ok(Err(SubmissionLogicError::NonceTooLow))
										},
										// This occurs when the nonce has already been *consumed* i.e a transaction with
										// that nonce is in a block
										jsonrpsee::core::Error::Call(jsonrpsee::types::error::CallError::Custom(ref obj))
											if obj == &invalid_err_obj(InvalidTransaction::Stale) =>
										{
											break Ok(Err(SubmissionLogicError::NonceTooLow))
										},
										jsonrpsee::core::Error::Call(jsonrpsee::types::error::CallError::Custom(ref obj))
											if obj == &invalid_err_obj(InvalidTransaction::BadProof) =>
										{
											/*slog::warn!(
												logger,
												"Extrinsic submission failed with nonce: {}. Error: {:?}. Refetching the runtime version.",
												account_nonce,
												rpc_err
											);*/
											let new_runtime_version = self.base_rpc_client.runtime_version().await?;
											if new_runtime_version == self.runtime_version {
												// slog::warn!(logger, "Fetched RuntimeVersion of {:?} is the same as the previous RuntimeVersion. This is not expected.", &runtime_version);
												// break, as the error is now very unlikely to be solved by fetching
												// again
												return Err(anyhow!("Fetched RuntimeVersion of {:?} is the same as the previous RuntimeVersion. This is not expected.", self.runtime_version))
											}

											self.runtime_version = new_runtime_version;
										},
										err => break Err(err.into()),
									}
								}
							}
						}

						async fn submit_extrinsic(&mut self, call: state_chain_runtime::RuntimeCall, genesis_hash: H256, latest_block_hash: H256, latest_block_number: state_chain_runtime::BlockNumber, request_id: ExtrinsicRequestID) -> Result<(), anyhow::Error> {
							loop {
								match self.try_submit_extrinsic(call.clone(), self.anticipated_nonce, genesis_hash, latest_block_hash, latest_block_number, request_id).await? {
									Ok(()) => {
										self.anticipated_nonce += 1;
										break
									},
									Err(SubmissionLogicError::NonceTooLow) => {
										self.anticipated_nonce += 1;
									}
								}
							}

							Ok(())
						}
					}

					let mut signed_extrinsic_submitter = SignedExtrinsicSubmitter::new(signer, finalized_nonce, runtime_version, base_rpc_client.clone());
					let mut next_request_id: ExtrinsicRequestID = 0;
					let mut extrinsic_requests: BTreeMap<ExtrinsicRequestID, ExtrinsicRequest> = Default::default();

					loop {
						tokio::select! {
							Some((call, result_sender)) = signed_extrinsic_request_receiver.recv() => {
								signed_extrinsic_submitter.submit_extrinsic(
									call.clone(),
									genesis_hash,
									latest_block_hash,
									latest_block_number,
									next_request_id,
								).await?;
								extrinsic_requests.insert(
									next_request_id,
									ExtrinsicRequest {
										submissions: 1,
										failed_submissions: 0,
										lifetime: ..=(latest_block_number+128),
										allow_unknown_finalized_status_on_death: true,
										call,
										result_sender: todo!()
									}
								);
								next_request_id += 1;
							},
							Ok(current_block_header) = signed_extrinsic_consumer_block_receiver.recv() => {
								let current_block_hash = current_block_header.hash();
								let current_block = base_rpc_client.block(current_block_hash).await?.unwrap().block;
								let current_events = base_rpc_client.storage_value::<frame_system::Events::<state_chain_runtime::Runtime>>(current_block_hash).await?;

								let current_finalized_nonce = base_rpc_client
									.storage_map_entry::<frame_system::Account<state_chain_runtime::Runtime>>(
										current_block_hash,
										&signed_extrinsic_submitter.signer.account_id,
									)
									.await?
									.nonce;

								if current_finalized_nonce < signed_extrinsic_submitter.finalized_nonce {
									return Err(anyhow!("Extrinsic signer's account was reaped"))
								} else {
									signed_extrinsic_submitter.finalized_nonce = current_finalized_nonce;
									signed_extrinsic_submitter.anticipated_nonce = state_chain_runtime::Index::max(
										signed_extrinsic_submitter.anticipated_nonce,
										current_finalized_nonce
									);

									for (extrinsic_index, extrinsic_events) in current_events.iter().filter_map(|event_record| {
										match &**event_record {
											frame_system::EventRecord { phase: frame_system::Phase::ApplyExtrinsic(extrinsic_index), event, .. } => Some((extrinsic_index, event)),
											_ => None
										}
									}).sorted_by_key(|(extrinsic_index, _)| *extrinsic_index).group_by(|(extrinsic_index, _)| *extrinsic_index).into_iter() {
										let extrinsic = &current_block.extrinsics[*extrinsic_index as usize];
										if let Some((address, _, extra)) = &extrinsic.signature {
											if *address == MultiAddress::Id(signed_extrinsic_submitter.signer.account_id.clone()) { // Assumption needs checking
												if let Some(submissions) = signed_extrinsic_submitter.submissions_by_nonce.remove(&extra.5.0) {
													let tx_hash = <state_chain_runtime::Runtime as frame_system::Config>::Hashing::hash_of(extrinsic);

													// Send extrinsic request result if one of its submissions for this nonce was included in this block
													if let Some((submission, extrinsic_request)) = submissions.iter().find_map(|submission| {
														// Note: It is technically possible for a hash collision to occur, but it is so unlikely it is effectively impossible. If it where to occur this code would not notice the included extrinsic was not actually the requested one, but otherwise would continue to work.
														if submission.tx_hash == tx_hash {
															extrinsic_requests
																.remove(&submission.request_id)
																.map(|pending_extrinsic_request| (submission, pending_extrinsic_request))
														} else {
															None
														}
													}) {
														assert!(submission.lifetime.contains(&current_block_header.number));

														let extrinsic_events = extrinsic_events.map(|(_extrinsics_index, event)| event.clone()).collect::<Vec<_>>();
														let _result = extrinsic_request.result_sender.send({
															match extrinsic_events.iter().find_map(|event| match event {
																state_chain_runtime::RuntimeEvent::System(frame_system::Event::ExtrinsicSuccess { dispatch_info }) => {
																	Some(Ok(dispatch_info))
																},
																state_chain_runtime::RuntimeEvent::System(frame_system::Event::ExtrinsicFailed { dispatch_error, dispatch_info }) => {
																	Some(Err((dispatch_info, dispatch_error)))
																},
																_ => None
															}).unwrap() {
																Ok(dispatch_info) => Ok((dispatch_info.clone(), extrinsic_events)),
																Err((dispatch_info, dispatch_error)) => Err(ExtrinsicFailure::Finalized(dispatch_info.clone(), dispatch_error.clone(), extrinsic_events)),
															}
														});
													}

													for submission in submissions {
														if let Some(extrinsic_request) = extrinsic_requests.get_mut(&submission.request_id) {
															extrinsic_request.failed_submissions += 1;
														}
													}
												}
											}
										}
									}

									signed_extrinsic_submitter.submissions_by_nonce.retain(|nonce, submissions| {
										assert!(signed_extrinsic_submitter.finalized_nonce <= *nonce);

										submissions.retain(|submission| {
											let retain = submission.lifetime.contains(&(latest_block_number + 1));

											if !retain {
												if let Some(extrinsic_request) = extrinsic_requests.get_mut(&submission.request_id) {
													extrinsic_request.failed_submissions += 1;
												}
											}

											retain
										});

										!submissions.is_empty()
									});

									for (request_id, extrinsic_request) in extrinsic_requests.drain_filter(|request_id, extrinsic_request| {
										!extrinsic_request.lifetime.contains(&(latest_block_number + 1)) && (
											extrinsic_request.allow_unknown_finalized_status_on_death
											|| extrinsic_request.submissions == extrinsic_request.failed_submissions
										)
									}) {
										let _result = extrinsic_request.result_sender.send(Err(ExtrinsicFailure::TimedOut(
											if extrinsic_request.submissions == extrinsic_request.failed_submissions {
												NonFinalizedStatus::Guaranteed
											} else {
												NonFinalizedStatus::Unknown
											}
										)));
									}

									for (request_id, extrinsic_request) in &mut extrinsic_requests {
										if extrinsic_request.submissions == extrinsic_request.failed_submissions {
											signed_extrinsic_submitter.submit_extrinsic(extrinsic_request.call.clone(), genesis_hash, latest_block_hash, latest_block_number, *request_id).await?;
											extrinsic_request.submissions += 1;
										}
									}

									// Handle possibility of stuck nonce caused submissions being dropped from the mempool or broken submissions either submitted here or externally
									{
										let mut shuffled_extrinsic_requests = {
											use rand::prelude::SliceRandom;
											let mut extrinsic_requests = extrinsic_requests.iter_mut().collect::<Vec<_>>();
											extrinsic_requests.shuffle(&mut rand::thread_rng());
											extrinsic_requests.into_iter()
										};

										if let Some((request_id, extrinsic_request)) = shuffled_extrinsic_requests.next() {
											// TODO: Consider using pending_extrinsics rpc call instead to do this
											match signed_extrinsic_submitter.try_submit_extrinsic(extrinsic_request.call.clone(), signed_extrinsic_submitter.finalized_nonce, genesis_hash, latest_block_hash, latest_block_number, *request_id).await? {
												Ok(()) => {
													// log warning
													signed_extrinsic_submitter.anticipated_nonce = signed_extrinsic_submitter.finalized_nonce;
													for (request_id, extrinsic_request) in shuffled_extrinsic_requests {
														match signed_extrinsic_submitter.try_submit_extrinsic(extrinsic_request.call.clone(), signed_extrinsic_submitter.anticipated_nonce, genesis_hash, latest_block_hash, latest_block_number, *request_id).await? {
															Ok(()) => signed_extrinsic_submitter.anticipated_nonce += 1,
															Err(SubmissionLogicError::NonceTooLow) => break
														}
													}
												},
												Err(SubmissionLogicError::NonceTooLow) => {} // expected, ignore
											}
										}
									}
								}
							}
							else => break Ok(())
						}
					}
				}
			}),
			_unsigned_extrinsic_consumer: scope.spawn_with_handle({
				let logger = logger.clone();
				let base_rpc_client = base_rpc_client.clone();

				async move {
					while let Some((call, result_sender)) = unsigned_extrinsic_request_receiver.recv().await {
						let _result = result_sender.send({
							let extrinsic = state_chain_runtime::UncheckedExtrinsic::new_unsigned(call.clone());
							let expected_hash = sp_runtime::traits::BlakeTwo256::hash_of(&extrinsic);
							match base_rpc_client.submit_extrinsic(extrinsic).await {
								Ok(tx_hash) => {
									slog::info!(
										logger,
										"Unsigned extrinsic {:?} submitted successfully with tx_hash: {:#x}",
										&call,
										tx_hash
									);
									assert_eq!(
										tx_hash, expected_hash,
										"tx_hash returned from RPC does not match expected hash"
									);
									Ok(tx_hash)
								},
								Err(rpc_err) => {
									match rpc_err {
										// POOL_ALREADY_IMPORTED error occurs when the transaction is already in the
										// pool More than one node can submit the same unsigned extrinsic. E.g. in the
										// case of a threshold signature success. Thus, if we get a "Transaction already
										// in pool" "error" we know that this particular extrinsic has already been
										// submitted. And so we can ignore the error and return the transaction hash
										jsonrpsee::core::Error::Call(jsonrpsee::types::error::CallError::Custom(ref obj)) if obj.code() == 1013 => {
											slog::trace!(
												logger,
												"Unsigned extrinsic {:?} with tx_hash {:#x} already in pool.",
												&call,
												expected_hash
											);
											Ok(expected_hash)
										},
										_ => {
											slog::error!(
												logger,
												"Unsigned extrinsic failed with error: {}. Extrinsic: {:?}",
												rpc_err,
												&call
											);
											Err(rpc_err.into())
										},
									}
								},
							}
						});
					}

					Ok(())
				}
			}),
			_block_producer: scope.spawn_with_handle({
				async move {
					while let Ok(_ok) = block_sender.broadcast(finalized_block_header_stream.next().await.unwrap()?).await {}
					Ok(())
				}
			}),
			base_rpc_client,
		});

		slog::info!(
			logger,
			"Initialised StateChainClient at block `{}`; block hash: `{:#x}`",
			latest_block_number,
			latest_block_hash
		);

		Ok((latest_block_hash, block_receiver, state_chain_client))
	}
}

#[cfg(test)]
pub mod mocks {
	use crate::state_chain_observer::client::{
		extrinsic_api::ExtrinsicApi, storage_api::StorageApi,
	};
	use anyhow::Result;
	use async_trait::async_trait;
	use frame_support::storage::types::QueryKindTrait;
	use futures::Stream;
	use jsonrpsee::core::RpcResult;
	use mockall::mock;
	use sp_core::{storage::StorageKey, H256};
	use state_chain_runtime::AccountId;

	use super::storage_api::{
		StorageDoubleMapAssociatedTypes, StorageMapAssociatedTypes, StorageValueAssociatedTypes,
	};

	mock! {
		pub StateChainClient {}
		#[async_trait]
		impl ExtrinsicApi for StateChainClient {
			fn account_id(&self) -> AccountId;

			async fn submit_signed_extrinsic<Call>(
				&self,
				call: Call,
				logger: &slog::Logger,
			) -> Result<H256>
			where
				Call: Into<state_chain_runtime::RuntimeCall> + Clone + std::fmt::Debug + Send + Sync + 'static;

			async fn submit_unsigned_extrinsic<Call>(
				&self,
				call: Call,
				logger: &slog::Logger,
			) -> Result<H256>
			where
				Call: Into<state_chain_runtime::RuntimeCall> + Clone + std::fmt::Debug + Send + Sync + 'static;

			async fn watch_submitted_extrinsic<BlockStream>(
				&self,
				extrinsic_hash: state_chain_runtime::Hash,
				block_stream: &mut BlockStream,
			) -> Result<Vec<state_chain_runtime::RuntimeEvent>>
			where
				BlockStream:
					Stream<Item = state_chain_runtime::Header> + Unpin + Send + 'static;
		}
		#[async_trait]
		impl StorageApi for StateChainClient {
			async fn storage_item<
				Value: codec::FullCodec + 'static,
				OnEmpty: 'static,
				QueryKind: QueryKindTrait<Value, OnEmpty> + 'static,
			>(
				&self,
				storage_key: StorageKey,
				block_hash: state_chain_runtime::Hash,
			) -> RpcResult<<QueryKind as QueryKindTrait<Value, OnEmpty>>::Query>;

			async fn storage_value<StorageValue: StorageValueAssociatedTypes + 'static>(
				&self,
				block_hash: state_chain_runtime::Hash,
			) -> RpcResult<<StorageValue::QueryKind as QueryKindTrait<StorageValue::Value, StorageValue::OnEmpty>>::Query>;

			async fn storage_map_entry<StorageMap: StorageMapAssociatedTypes + 'static>(
				&self,
				block_hash: state_chain_runtime::Hash,
				key: &StorageMap::Key,
			) -> RpcResult<
				<StorageMap::QueryKind as QueryKindTrait<StorageMap::Value, StorageMap::OnEmpty>>::Query,
			>
			where
				StorageMap::Key: Sync;

			async fn storage_double_map_entry<StorageDoubleMap: StorageDoubleMapAssociatedTypes + 'static>(
				&self,
				block_hash: state_chain_runtime::Hash,
				key1: &StorageDoubleMap::Key1,
				key2: &StorageDoubleMap::Key2,
			) -> RpcResult<
				<StorageDoubleMap::QueryKind as QueryKindTrait<
					StorageDoubleMap::Value,
					StorageDoubleMap::OnEmpty,
				>>::Query,
			>
			where
				StorageDoubleMap::Key1: Sync,
				StorageDoubleMap::Key2: Sync;

			async fn storage_map<StorageMap: StorageMapAssociatedTypes + 'static>(
				&self,
				block_hash: state_chain_runtime::Hash,
			) -> RpcResult<Vec<(<StorageMap as StorageMapAssociatedTypes>::Key, StorageMap::Value)>>;
		}
	}
}
