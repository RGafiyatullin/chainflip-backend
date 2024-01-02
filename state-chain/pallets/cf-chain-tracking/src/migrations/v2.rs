use crate::*;
use sp_std::marker::PhantomData;

pub struct Migration<T: Config<I>, I: 'static>(PhantomData<(T, I)>);

impl<T: Config<I>, I: 'static> OnRuntimeUpgrade for Migration<T, I> {
	fn on_runtime_upgrade() -> Weight {
        // storage::migration::
		unimplemented!()
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

    pub type BtcAmount = u64;

    #[derive(
        Copy,
        Clone,
        RuntimeDebug,
        PartialEq,
        Eq,
        Encode,
        Decode,
        MaxEncodedLen,
        TypeInfo,
    )]
    pub struct BitcoinFeeInfo {
        pub fee_per_input_utxo: BtcAmount,
        pub fee_per_output_utxo: BtcAmount,
        pub min_fee_required_per_tx: BtcAmount,
    }
}
