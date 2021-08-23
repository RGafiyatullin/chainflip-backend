#![cfg_attr(not(feature = "std"), no_std)]
//! Request-Reply Pallet
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{Parameter, dispatch::DispatchResult};
pub use pallet::*;
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;

pub trait RequestResponse<T: frame_system::Config> {
	type Response: Parameter;

	fn on_response(&self, _response: Self::Response) -> DispatchResult;
}

pub trait BaseConfig: frame_system::Config {
	/// The id type used to identify individual signing keys.
	type KeyId: Parameter;
	type ValidatorId: Parameter;
	type ChainId: Parameter;
}

// These would be defined in their own modules but adding it here for now.
// Macros might help reduce the boilerplat but I don't think it's too bad.
pub mod instances {
	pub use super::*;
	use codec::{Decode, Encode};

	// A signature request.
	pub mod signing {
		use super::*;

		#[derive(Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode)]
		pub struct Response<T: BaseConfig> {
			signing_key: T::KeyId,
			payload: Vec<u8>,
			signatories: Vec<T::ValidatorId>,
		}
		
		#[derive(Clone, PartialEq, Eq, Encode, Decode)]
		pub enum Reply<T: BaseConfig> {
			Success { sig: Vec<u8> },
			Failure { bad_nodes: Vec<T::ValidatorId> },
		}

		impl<T: BaseConfig> sp_std::fmt::Debug for Reply<T> {
			fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
				f.write_str(stringify!(Reply))
			}
		}

		impl<T: BaseConfig> RequestResponse<T> for Response<T> {
			type Response = Reply<T>;

			fn on_response(&self, _response: Self::Response) -> DispatchResult {
				todo!("The implementing pallet could store the result, or process a claim, or whatever.")
			}
		}
	}

	// A broadcast request.
	pub mod broadcast {
		use super::*;

		#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
		pub struct Request<T: BaseConfig> {
			chain: T::ChainId,
		}

		#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
		pub enum Response {
			Success,
			Failure,
			Timeout,
		}

		impl<T: BaseConfig> RequestResponse<T> for Request<T> {
			type Response = Response;

			fn on_response(&self, _response: Self::Response) -> DispatchResult {
				todo!("Handle failure and timeouts.")
			}
		}
	}
}

pub type RequestId = u64;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{Twox64Concat, dispatch::DispatchResultWithPostInfo};
	use frame_system::pallet_prelude::*;
	use frame_support::pallet_prelude::*;
	use codec::FullCodec;

	type ResponseFor<T, I> = <<T as Config<I>>::Request as RequestResponse<T>>::Response;

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config<I: 'static = ()>: BaseConfig {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

		/// The request-response definition for this instance.
		type Request: RequestResponse<Self> + Member + FullCodec;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::storage]
	#[pallet::getter(fn request_id_counter)]
	pub type RequestIdCounter<T, I = ()> = StorageValue<_, RequestId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pending_request)]
	pub type PendingRequests<T: Config<I>, I: 'static = ()> = StorageMap<_, Twox64Concat, RequestId, T::Request, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// An outgoing request. [id, request]
		Request(RequestId, T::Request),
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// The provided request id is invalid.
		InvalidRequestId,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Reply.
		#[pallet::weight(10_000)]
		pub fn response(origin: OriginFor<T>, id: RequestId, response: ResponseFor<T, I>) -> DispatchResultWithPostInfo {
			// Probably needs to be witnessed.
			let _who = ensure_signed(origin)?;
			
			// 1. Pull the request type out of storage.
			let request = PendingRequests::<T, I>::take(id).ok_or(Error::<T, I>::InvalidRequestId)?;

			// 2. Dispatch the callback.
			let _ = request.on_response(response)?;

			Ok(().into())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Emits a request event, stores it, and returns its id.
	pub fn request(request: T::Request) -> u64 {
		// Get a new id.
		let id = RequestIdCounter::<T, I>::mutate(|id| { *id += 1; *id });

		// Store the request.
		PendingRequests::<T, I>::insert(id, &request);

		// Emit the request to the CFE.
		Self::deposit_event(Event::<T, I>::Request(id, request));

		id
	}
}
