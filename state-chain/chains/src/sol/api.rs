use cf_primitives::chains::Solana;
use codec::{Decode, Encode};
use frame_support::{
	sp_runtime::DispatchError, CloneNoBound, DebugNoBound, EqNoBound, Never, PartialEqNoBound,
};
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, vec};

use super::SolanaCrypto;

#[derive(CloneNoBound, DebugNoBound, PartialEqNoBound, EqNoBound, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(Environment))]
pub enum SolanaApi<Environment: 'static> {
	#[doc(hidden)]
	#[codec(skip)]
	_Phantom(PhantomData<Environment>, Never),
}

use crate::{
	AllBatch, ApiCall, ChainCrypto, ConsolidateCall, ExecutexSwapAndCall, SetAggKeyWithAggKey,
	TransferFallback,
};

impl<Env> SetAggKeyWithAggKey<SolanaCrypto> for SolanaApi<Env> {
	fn new_unsigned(
		_maybe_old_key: Option<<SolanaCrypto as ChainCrypto>::AggKey>,
		_new_key: <SolanaCrypto as ChainCrypto>::AggKey,
	) -> Result<Self, crate::SetAggKeyWithAggKeyError> {
		unimplemented!()
	}
}

impl<Env> ApiCall<SolanaCrypto> for SolanaApi<Env> {
	fn threshold_signature_payload(&self) -> <SolanaCrypto as ChainCrypto>::Payload {
		unimplemented!()
	}

	fn signed(
		self,
		_threshold_signature: &<SolanaCrypto as ChainCrypto>::ThresholdSignature,
	) -> Self {
		unimplemented!()
	}

	fn chain_encoded(&self) -> vec::Vec<u8> {
		unimplemented!()
	}

	fn is_signed(&self) -> bool {
		unimplemented!()
	}

	fn transaction_out_id(&self) -> <SolanaCrypto as ChainCrypto>::TransactionOutId {
		unimplemented!()
	}
}

impl<Env> ConsolidateCall<Solana> for SolanaApi<Env> {
	fn consolidate_utxos() -> Result<Self, crate::ConsolidationError> {
		unimplemented!()
	}
}

impl<Env> TransferFallback<Solana> for SolanaApi<Env> {
	fn new_unsigned(
		_transfer_param: crate::TransferAssetParams<Solana>,
	) -> Result<Self, DispatchError> {
		unimplemented!()
	}
}

impl<Env> ExecutexSwapAndCall<Solana> for SolanaApi<Env> {
	fn new_unsigned(
		_transfer_param: crate::TransferAssetParams<Solana>,
		_source_chain: cf_primitives::ForeignChain,
		_source_address: Option<crate::ForeignChainAddress>,
		_gas_budget: <Solana as crate::Chain>::ChainAmount,
		_message: vec::Vec<u8>,
	) -> Result<Self, DispatchError> {
		unimplemented!()
	}
}

impl<Env> AllBatch<Solana> for SolanaApi<Env> {
	fn new_unsigned(
		_fetch_params: vec::Vec<crate::FetchAssetParams<Solana>>,
		_transfer_params: vec::Vec<crate::TransferAssetParams<Solana>>,
	) -> Result<Self, crate::AllBatchError> {
		unimplemented!()
	}
}
