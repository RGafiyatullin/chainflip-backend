use cf_primitives::BroadcastId;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::{ConstBool, RuntimeDebug};
use sp_std::vec::Vec;

use crate::ChainCrypto;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolanaCrypto;

#[derive(
	Copy,
	Clone,
	RuntimeDebug,
	Default,
	PartialEq,
	Eq,
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
	Ord,
	PartialOrd,
	serde::Serialize,
	serde::Deserialize,
)]
pub struct Stub;

impl ChainCrypto for SolanaCrypto {
	type UtxoChain = ConstBool<false>;

	type AggKey = Stub;
	type Payload = Stub;
	type ThresholdSignature = Stub;
	type TransactionInId = Stub;
	type TransactionOutId = Stub;
	type GovKey = Stub;

	fn verify_threshold_signature(
		_agg_key: &Self::AggKey,
		_payload: &Self::Payload,
		_signature: &Self::ThresholdSignature,
	) -> bool {
		unimplemented!()
	}

	fn agg_key_to_payload(_agg_key: Self::AggKey, _for_handover: bool) -> Self::Payload {
		unimplemented!()
	}

	fn handover_key_matches(_current_key: &Self::AggKey, _new_key: &Self::AggKey) -> bool {
		unimplemented!()
	}

	fn key_handover_is_required() -> bool {
		unimplemented!()
	}

	fn maybe_broadcast_barriers_on_rotation(
		_rotation_broadcast_id: BroadcastId,
	) -> Vec<BroadcastId> {
		unimplemented!()
	}
}
