use crate::*;
use sp_std::marker::PhantomData;

pub struct Migration<T: Config<I>, I: 'static>(PhantomData<(T, I)>);

#[cfg(feature = "try-runtime")]
use sp_std::prelude::Vec;

impl<T, I> OnRuntimeUpgrade for Migration<T, I>
where
	T: Config<I>,
	I: 'static,
	ChainState<T::TargetChain>: v1::FromV1,
{
	fn on_runtime_upgrade() -> Weight {
		// Runtime-check: only migrate the Bitcoin TrackedData
		if T::TargetChain::NAME == cf_chains::Bitcoin::NAME {
			// Compile-time: `impl v1::FromV1 for ChainState<Chain>`
			//     should be defined for every `Chain` we use this migration with.
			let translated_opt =
				CurrentChainState::<T, I>::translate(|old| old.map(v1::FromV1::from_v1))
					// XXX: is it okay to panic here? How to signalise an error otherwise?
					.expect("failed to decode v1-storage");
			if let Some(translated) = translated_opt {
				CurrentChainState::<T, I>::put(translated);
			}
		}
		// For the chains other than Bitcoin `v1::FromV1` should be defined as
		//    something that explodes rather than silently corrupts the data.

		Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
		unimplemented!()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), DispatchError> {
		unimplemented!()
	}
}

mod v1 {
	use crate::*;

	pub trait FromV1 {
		type OldType: Decode;
		fn from_v1(old: Self::OldType) -> Self;
	}

	#[derive(Copy, Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub enum Never {}

	macro_rules! impl_unreachable_from_v1_for_chain {
		($chain: ty) => {
			impl FromV1 for crate::ChainState<$chain> {
				type OldType = Never;
				fn from_v1(old: Self::OldType) -> Self {
					unreachable!(
						"We are not supposed to have an instance of {}",
						core::any::type_name::<Self::OldType>()
					)
				}
			}
		};
	}
	impl_unreachable_from_v1_for_chain!(cf_chains::Ethereum);
	impl_unreachable_from_v1_for_chain!(cf_chains::Polkadot);

	pub mod btc {
		use super::FromV1;
		use crate::*;

		pub type BtcBlockNumber = u64;
		pub type BtcAmount = u64;

		#[derive(
			Copy, Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo,
		)]
		pub struct FeeInfo {
			pub fee_per_input_utxo: BtcAmount,
			pub fee_per_output_utxo: BtcAmount,
			pub min_fee_required_per_tx: BtcAmount,
		}

		#[derive(
			Copy, Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo,
		)]
		pub struct TrackedData {
			pub block_height: BtcBlockNumber,
			pub tracked_data: FeeInfo,
		}

		impl FromV1 for ChainState<cf_chains::Bitcoin> {
			type OldType = TrackedData;

			fn from_v1(TrackedData { block_height, tracked_data }: TrackedData) -> Self {
                log::warn!("upgrading @{:?}", block_height);
				unimplemented!()
			}
		}
	}
}
