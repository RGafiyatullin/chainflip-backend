#![cfg_attr(not(feature = "std"), no_std)]
// This can be removed after rustc version 1.53.
#![feature(int_bits_const)]

//! Transaction Broadcast Pallet
//! https://swimlanes.io/u/1s-nyDuYQ

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use cf_chains::Chain;
use cf_traits::{offline_conditions::*, Chainflip, SignerNomination};
use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResultWithPostInfo, Parameter, Twox64Concat, traits::Get};
use frame_system::pallet_prelude::OriginFor;
pub use pallet::*;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum BroadcastFailure {
	/// The transaction was rejected because of some user error.
	TransactionRejected,
	/// The transaction failed for some unknown reason.
	TransactionFailed,
	/// The transaction stalled.
	TransactionTimeout,
}

/// The [TransactionContext] should contain all the state required to construct and process transactions for a given
/// chain.
pub trait BroadcastConfig<T: Chainflip> {
	/// A chain identifier.
	type Chain: Chain;
	/// An unsigned version of the transaction that needs to signed before it can be broadcast.
	type UnsignedTransaction: Parameter;
	/// A transaction that has been signed by some account and is ready to be broadcast.
	type SignedTransaction: Parameter;
	/// The transaction hash type used to uniquely identify signed transactions.
	type TransactionHash: Parameter;

	/// Verify the signed transaction when it is submitted to the state chain by the nominated signer.
	fn verify_transaction(
		signer: &T::ValidatorId,
		unsigned_tx: &Self::UnsignedTransaction,
		signed_tx: &Self::SignedTransaction,
	) -> Option<()>;
}

/// A unique id for each broadcast attempt.
pub type BroadcastAttemptId = u64;

/// A unique id for each broadcast.
pub type BroadcastId = u32;

/// The number of broadcast attempts that were made before this one.
pub type AttemptCount = u32;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{ensure, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	pub type SignedTransactionFor<T, I> =
		<<T as Config<I>>::BroadcastConfig as BroadcastConfig<T>>::SignedTransaction;
	pub type UnsignedTransactionFor<T, I> =
		<<T as Config<I>>::BroadcastConfig as BroadcastConfig<T>>::UnsignedTransaction;
	pub type TransactionHashFor<T, I> =
		<<T as Config<I>>::BroadcastConfig as BroadcastConfig<T>>::TransactionHash;

	#[derive(Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode)]
	pub struct SigningAttempt<T: Config<I>, I: 'static> {
		pub broadcast_id: BroadcastId,
		pub attempt_count: AttemptCount,
		pub unsigned_tx: UnsignedTransactionFor<T, I>,
		pub nominee: T::ValidatorId,
	}

	#[derive(Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode)]
	pub struct BroadcastAttempt<T: Config<I>, I: 'static> {
		pub broadcast_id: BroadcastId,
		pub attempt_count: AttemptCount,
		pub unsigned_tx: UnsignedTransactionFor<T, I>,
		pub signer: T::ValidatorId,
		pub signed_tx: SignedTransactionFor<T, I>,
	}

	#[derive(Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode)]
	pub struct FailedAttempt<T: Config<I>, I: 'static> {
		pub broadcast_id: BroadcastId,
		pub attempt_count: AttemptCount,
		pub unsigned_tx: UnsignedTransactionFor<T, I>,
	}

	impl<T: Config<I>, I: 'static> From<BroadcastAttempt<T, I>> for FailedAttempt<T, I> {
		fn from(failed: BroadcastAttempt<T, I>) -> Self {
			Self {
				broadcast_id: failed.broadcast_id,
				attempt_count: failed.attempt_count,
				unsigned_tx: failed.unsigned_tx,
			}
		}
	}

	impl<T: Config<I>, I: 'static> From<SigningAttempt<T, I>> for FailedAttempt<T, I> {
		fn from(failed: SigningAttempt<T, I>) -> Self {
			Self {
				broadcast_id: failed.broadcast_id,
				attempt_count: failed.attempt_count,
				unsigned_tx: failed.unsigned_tx,
			}
		}
	}

	#[derive(Copy, Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode)]
	pub enum SigningOrBroadcast {
		SigningStage,
		BroadcastStage,
	}

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config<I: 'static = ()>: Chainflip {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

		/// A marker trait identifying the chain that we are broadcasting to.
		type TargetChain: Chain;

		/// The broadcast configuration for this instance.
		type BroadcastConfig: BroadcastConfig<Self, Chain = Self::TargetChain>;

		/// Signer nomination.
		type SignerNomination: SignerNomination<SignerId = Self::ValidatorId>;

		/// For reporting bad actors.
		type OfflineReporter: OfflineReporter<ValidatorId = Self::ValidatorId>;

		/// The timeout duration for the signing stage, measured in number of blocks.
		#[pallet::constant]
		type SigningTimeout: Get<BlockNumberFor<Self>>;

		/// The timeout duration for the broadcast stage, measured in number of blocks.
		#[pallet::constant]
		type BroadcastTimeout: Get<BlockNumberFor<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::storage]
	pub type BroadcastAttemptIdCounter<T, I = ()> = StorageValue<_, BroadcastAttemptId, ValueQuery>;

	#[pallet::storage]
	pub type BroadcastIdCounter<T, I = ()> = StorageValue<_, BroadcastId, ValueQuery>;

	#[pallet::storage]
	pub type AwaitingSignature<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, BroadcastAttemptId, SigningAttempt<T, I>, OptionQuery>;

	#[pallet::storage]
	pub type AwaitingBroadcast<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, BroadcastAttemptId, BroadcastAttempt<T, I>, OptionQuery>;

	#[pallet::storage]
	pub type RetryQueue<T: Config<I>, I: 'static = ()> =
		StorageValue<_, Vec<FailedAttempt<T, I>>, ValueQuery>;

	#[pallet::storage]
	pub type Expiries<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		T::BlockNumber,
		Vec<(SigningOrBroadcast, BroadcastAttemptId)>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// [broadcast_attempt_id, validator_id, unsigned_tx]
		TransactionSigningRequest(
			BroadcastAttemptId,
			T::ValidatorId,
			UnsignedTransactionFor<T, I>,
		),
		/// [broadcast_attempt_id, signed_tx]
		BroadcastRequest(BroadcastAttemptId, SignedTransactionFor<T, I>),
		/// [broadcast_id]
		BroadcastComplete(BroadcastId),
		/// [broadcast_id, attempt]
		RetryScheduled(BroadcastId, AttemptCount),
		/// [broadcast_id, attempt, failed_transaction]
		BroadcastFailed(BroadcastId, AttemptCount, UnsignedTransactionFor<T, I>),
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// The provided request id is invalid.
		InvalidBroadcastId,
		/// The transaction signer is not signer who was nominated.
		InvalidSigner,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_initialize(block_number: BlockNumberFor<T>) -> frame_support::weights::Weight {
			let retries = RetryQueue::<T, I>::take();
			let retry_count = retries.len();
			for failed in retries {
				Self::retry_failed_broadcast(failed);
			}

			let expiries = Expiries::<T, I>::take(block_number);
			for (stage, id) in expiries.iter() {
				match stage {
					SigningOrBroadcast::SigningStage => {
						AwaitingSignature::<T, I>::take(id).map(|signing_attempt| {
							Self::retry_failed_broadcast(signing_attempt.into());
						});
					}
					SigningOrBroadcast::BroadcastStage => {
						AwaitingBroadcast::<T, I>::take(id).map(|broadcast_attempt| {
							Self::retry_failed_broadcast(broadcast_attempt.into());
						});
					}
				}
			}

			// TODO: replace this with benchmark results.
			retry_count as u64
				* frame_support::weights::RuntimeDbWeight::default().reads_writes(3, 3)
				+ expiries.len() as u64
					* frame_support::weights::RuntimeDbWeight::default().reads_writes(1, 1)
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Begin the process of broadcasting a transaction.
		///
		/// This is the first step - requsting a transaction signature from a nominated validator.
		#[pallet::weight(10_000)]
		pub fn start_sign_and_broadcast(
			origin: OriginFor<T>,
			unsigned_tx: UnsignedTransactionFor<T, I>,
		) -> DispatchResultWithPostInfo {
			// TODO: This doesn't necessarily have to be witnessed, but *should* be restricted such that it can only
			// be called internally.
			let _ = T::EnsureWitnessed::ensure_origin(origin)?;

			let broadcast_id = BroadcastIdCounter::<T, I>::mutate(|id| {
				*id += 1;
				*id
			});

			Self::start_broadcast_attempt(broadcast_id, 0, unsigned_tx);

			Ok(().into())
		}

		/// Called by the nominated signer when they have completed and signed the transaction, and it is therefore ready
		/// to be broadcast. The signed transaction is stored on-chain so that any node can potentially broadcast it to
		/// the target chain. Emits an event that will trigger the broadcast to the target chain.
		#[pallet::weight(10_000)]
		pub fn transaction_ready(
			origin: OriginFor<T>,
			attempt_id: BroadcastAttemptId,
			signed_tx: SignedTransactionFor<T, I>,
		) -> DispatchResultWithPostInfo {
			let signer = ensure_signed(origin)?;

			let signing_attempt =
				AwaitingSignature::<T, I>::get(attempt_id).ok_or(Error::<T, I>::InvalidBroadcastId)?;

			ensure!(
				signing_attempt.nominee == signer.into(),
				Error::<T, I>::InvalidSigner
			);

			AwaitingSignature::<T, I>::remove(attempt_id);

			if T::BroadcastConfig::verify_transaction(
				&signing_attempt.nominee,
				&signing_attempt.unsigned_tx,
				&signed_tx,
			)
			.is_some()
			{
				Self::deposit_event(Event::<T, I>::BroadcastRequest(attempt_id, signed_tx.clone()));
				AwaitingBroadcast::<T, I>::insert(
					attempt_id,
					BroadcastAttempt {
						broadcast_id: signing_attempt.broadcast_id,
						unsigned_tx: signing_attempt.unsigned_tx,
						signer: signing_attempt.nominee.clone(),
						signed_tx,
						attempt_count: signing_attempt.attempt_count,
					},
				);

				// Schedule expiry.
				let expiry_block = frame_system::Pallet::<T>::block_number() + T::BroadcastTimeout::get();
				Expiries::<T, I>::mutate(expiry_block, |entries| {
					entries.push((SigningOrBroadcast::BroadcastStage, attempt_id))
				});
			} else {
				Self::report_and_schedule_retry(
					&signing_attempt.nominee.clone(),
					signing_attempt.into(),
				)
			}

			Ok(().into())
		}

		/// Nodes have witnessed that the transaction has reached finality on the target chain.
		#[pallet::weight(10_000)]
		pub fn broadcast_success(
			origin: OriginFor<T>,
			attempt_id: BroadcastAttemptId,
			_tx_hash: TransactionHashFor<T, I>,
		) -> DispatchResultWithPostInfo {
			let _ = T::EnsureWitnessed::ensure_origin(origin)?;

			// Remove the broadcast now it's completed.
			let BroadcastAttempt::<T, I> { broadcast_id, .. } =
				AwaitingBroadcast::<T, I>::take(attempt_id).ok_or(Error::<T, I>::InvalidBroadcastId)?;

			Self::deposit_event(Event::<T, I>::BroadcastComplete(broadcast_id));

			Ok(().into())
		}

		/// Nodes have witnessed that something went wrong. The transaction may have been rejected outright or may
		/// have stalled on the target chain.
		#[pallet::weight(10_000)]
		pub fn broadcast_failure(
			origin: OriginFor<T>,
			attempt_id: BroadcastAttemptId,
			failure: BroadcastFailure,
			_tx_hash: TransactionHashFor<T, I>,
		) -> DispatchResultWithPostInfo {
			let _ = T::EnsureWitnessed::ensure_origin(origin)?;

			let failed_attempt =
				AwaitingBroadcast::<T, I>::take(attempt_id).ok_or(Error::<T, I>::InvalidBroadcastId)?;

			match failure {
				BroadcastFailure::TransactionRejected => {
					Self::report_and_schedule_retry(
						&failed_attempt.signer.clone(),
						failed_attempt.into(),
					);
				}
				BroadcastFailure::TransactionTimeout => {
					Self::schedule_retry(failed_attempt.into());
				}
				BroadcastFailure::TransactionFailed => {
					Self::deposit_event(Event::<T, I>::BroadcastFailed(
						failed_attempt.broadcast_id,
						failed_attempt.attempt_count,
						failed_attempt.unsigned_tx,
					));
				}
			};

			Ok(().into())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	fn start_broadcast_attempt(
		broadcast_id: BroadcastId,
		attempt_count: AttemptCount,
		unsigned_tx: UnsignedTransactionFor<T, I>,
	) {
		// Get a new id.
		let attempt_id = BroadcastAttemptIdCounter::<T, I>::mutate(|id| {
			*id += 1;
			*id
		});

		// Select a signer for this broadcast.
		let nominated_signer = T::SignerNomination::nomination_with_seed(attempt_id);

		AwaitingSignature::<T, I>::insert(
			attempt_id,
			SigningAttempt::<T, I> {
				broadcast_id,
				attempt_count,
				unsigned_tx: unsigned_tx.clone(),
				nominee: nominated_signer.clone(),
			},
		);

		// Schedule expiry.
		let expiry_block = frame_system::Pallet::<T>::block_number() + T::SigningTimeout::get();
		Expiries::<T, I>::mutate(expiry_block, |entries| {
			entries.push((SigningOrBroadcast::SigningStage, attempt_id))
		});

		// Emit the transaction signing request.
		Self::deposit_event(Event::<T, I>::TransactionSigningRequest(
			attempt_id,
			nominated_signer,
			unsigned_tx,
		));
	}

	fn report_and_schedule_retry(signer: &T::ValidatorId, failed: FailedAttempt<T, I>) {
		// TODO: set a sensible penalty and centralise. See #569
		const PENALTY: i32 = 0;
		T::OfflineReporter::report(OfflineCondition::ParticipateSigningFailed, PENALTY, signer)
			.unwrap_or_else(|_| {
				// Should never fail unless the validator doesn't exist.
				frame_support::debug::error!("Unable to report unknown validator {:?}", signer);
				0
			});
		Self::schedule_retry(failed);
	}

	fn schedule_retry(failed: FailedAttempt<T, I>) {
		RetryQueue::<T, I>::append(&failed);
		Self::deposit_event(Event::<T, I>::RetryScheduled(
			failed.broadcast_id,
			failed.attempt_count,
		));
	}

	fn retry_failed_broadcast(failed: FailedAttempt<T, I>) {
		Self::start_broadcast_attempt(
			failed.broadcast_id,
			failed.attempt_count.wrapping_add(1),
			failed.unsigned_tx,
		);
	}
}
