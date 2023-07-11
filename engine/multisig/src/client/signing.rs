mod signing_data;
mod signing_detail;
mod signing_stages;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::crypto::ChainSigning;

use super::common::{KeygenResult, SigningPayload};

pub use signing_data::{
	Comm1, LocalSig3, LocalSig3Inner, SigningCommitment, SigningData, VerifyComm2, VerifyLocalSig4,
};

pub use signing_detail::generate_schnorr_response;

pub use signing_stages::AwaitCommitments1;

#[cfg(test)]
pub use signing_data::{gen_signing_data_stage1, gen_signing_data_stage2, gen_signing_data_stage4};

pub use signing_detail::get_lagrange_coeff;

/// Payload and the key that should be used to sign over the payload
pub struct PayloadAndKey<C: ChainSigning> {
	pub payload: SigningPayload<C>,
	pub key: Arc<KeygenResult<C>>,
}

/// Data common for signing stages
pub struct SigningStateCommonInfo<C: ChainSigning> {
	pub payloads_and_keys: Vec<PayloadAndKey<C>>,
}

impl<C: ChainSigning> SigningStateCommonInfo<C> {
	pub fn payload_count(&self) -> usize {
		self.payloads_and_keys.len()
	}
}
