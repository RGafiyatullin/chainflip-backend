use std::fmt::Display;

use cf_primitives::AuthorityCount;
use serde::{Deserialize, Serialize};

use crate::{
	client::common::{
		BroadcastVerificationMessage, DelayDeserialization, PreProcessStageDataCheck,
		SigningStageName,
	},
	crypto::{ECPoint, MAX_POINT_SIZE, MAX_SCALAR_SIZE},
	ChainSigning, ChainTag, MAX_BTC_SIGNING_PAYLOADS,
};

#[cfg(test)]
pub use tests::{gen_signing_data_stage1, gen_signing_data_stage2, gen_signing_data_stage4};

/// Public components of the single-use nonces generated by
/// a single party at signer index `index`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]

pub struct SigningCommitment<P: ECPoint> {
	#[serde(bound = "")]
	pub d: P,
	#[serde(bound = "")]
	pub e: P,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Comm1Inner<P: ECPoint>(#[serde(bound = "")] pub Vec<SigningCommitment<P>>);

/// Calculate the size limit of the signing commitments. This scales with the number of payloads in
/// the ceremony.
const fn max_signing_commitments_size(number_of_payloads: usize) -> usize {
	// 2 points * payloads + length of vector
	2 * MAX_POINT_SIZE * number_of_payloads + 8
}

/// Calculate the size limit of the local sigs. This scales with the number of payloads in the
/// ceremony.
fn max_local_sigs_size(number_of_payloads: usize) -> usize {
	// 1 scalar * payloads + length of vector
	MAX_SCALAR_SIZE * number_of_payloads + 8
}

#[cfg(test)]
mod serialisation {
	use super::*;
	use crate::{
		client::helpers::{self, test_all_crypto_chains},
		CryptoScheme,
	};
	use rand::SeedableRng;

	fn test_signing_commitment_size_for_scheme<Chain: ChainSigning>() {
		let mut rng = rand::rngs::StdRng::from_seed([0u8; 32]);
		let comm1 = helpers::gen_dummy_signing_comm1::<
			<<Chain as ChainSigning>::CryptoScheme as CryptoScheme>::Point,
		>(&mut rng, 1);
		if matches!(<Chain as ChainSigning>::CHAIN_TAG, ChainTag::Ethereum) {
			// The constants are defined as to exactly match Ethereum/secp256k1,
			// which we demonstrate here:
			assert!(comm1.payload.len() == max_signing_commitments_size(1));
		} else {
			// Other chains might use a more compact serialization of primitives:
			assert!(comm1.payload.len() <= max_signing_commitments_size(1));
		}
	}

	#[test]
	fn test_signing_commitment_size() {
		test_all_crypto_chains!(test_signing_commitment_size_for_scheme());
	}

	fn test_local_sig_size_for_scheme<Chain: ChainSigning>() {
		let mut rng = rand::rngs::StdRng::from_seed([0u8; 32]);
		let sig = helpers::gen_dummy_local_sig::<
			<<Chain as ChainSigning>::CryptoScheme as CryptoScheme>::Point,
		>(&mut rng, 1);

		if matches!(<Chain as ChainSigning>::CHAIN_TAG, ChainTag::Ethereum) {
			// The constants are defined as to exactly match Ethereum/secp256k1,
			// which we demonstrate here:
			assert!(sig.payload.len() == max_local_sigs_size(1));
		} else {
			// Other chains might use a more compact serialization of primitives:
			assert!(sig.payload.len() <= max_local_sigs_size(1));
		}
	}

	#[test]
	fn test_local_sig_size() {
		test_all_crypto_chains!(test_local_sig_size_for_scheme());
	}
}

pub type Comm1<P> = DelayDeserialization<Comm1Inner<P>>;

pub type VerifyComm2<P> = BroadcastVerificationMessage<Comm1<P>>;

pub type LocalSig3<P> = DelayDeserialization<LocalSig3Inner<P>>;
pub type VerifyLocalSig4<P> = BroadcastVerificationMessage<LocalSig3<P>>;

/// Signature (the "response" part) shard generated by a single party
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalSig3Inner<P: ECPoint> {
	pub responses: Vec<P::Scalar>,
}

/// Data exchanged between parties during various stages
/// of the FROST signing protocol
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SigningData<P: ECPoint> {
	#[serde(bound = "")]
	CommStage1(Comm1<P>),
	#[serde(bound = "")]
	BroadcastVerificationStage2(VerifyComm2<P>),
	#[serde(bound = "")]
	LocalSigStage3(LocalSig3<P>),
	#[serde(bound = "")]
	VerifyLocalSigsStage4(VerifyLocalSig4<P>),
}

derive_impls_for_enum_variants!(impl<P: ECPoint> for Comm1<P>, SigningData::CommStage1, SigningData<P>);
derive_impls_for_enum_variants!(impl<P: ECPoint> for VerifyComm2<P>, SigningData::BroadcastVerificationStage2, SigningData<P>);
derive_impls_for_enum_variants!(impl<P: ECPoint> for LocalSig3<P>, SigningData::LocalSigStage3, SigningData<P>);
derive_impls_for_enum_variants!(impl<P: ECPoint> for VerifyLocalSig4<P>, SigningData::VerifyLocalSigsStage4, SigningData<P>);

derive_display_as_type_name!(Comm1<P: ECPoint>);
derive_display_as_type_name!(VerifyComm2<P: ECPoint>);
derive_display_as_type_name!(LocalSig3<P: ECPoint>);
derive_display_as_type_name!(VerifyLocalSig4<P: ECPoint>);

impl<P: ECPoint> Display for SigningData<P> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let inner = match self {
			SigningData::CommStage1(x) => x.to_string(),
			SigningData::BroadcastVerificationStage2(x) => x.to_string(),
			SigningData::LocalSigStage3(x) => x.to_string(),
			SigningData::VerifyLocalSigsStage4(x) => x.to_string(),
		};
		write!(f, "SigningData({inner})")
	}
}

impl<P: ECPoint> PreProcessStageDataCheck<SigningStageName> for SigningData<P> {
	fn is_data_size_valid<Chain: ChainSigning>(
		&self,
		num_of_parties: AuthorityCount,
		num_of_payloads: Option<usize>,
	) -> bool {
		let num_of_parties = num_of_parties as usize;
		match self {
			SigningData::CommStage1(_) => self.is_initial_stage_data_size_valid::<Chain>(),
			// It is safe to unwrap after the first stage because the number of payloads is always
			// known from then on (only for signing ceremonies)
			SigningData::BroadcastVerificationStage2(message) => message.is_data_size_valid(
				num_of_parties,
				max_signing_commitments_size(num_of_payloads.unwrap()),
			),
			SigningData::LocalSigStage3(message) =>
				message.payload.len() <= max_local_sigs_size(num_of_payloads.unwrap()),

			SigningData::VerifyLocalSigsStage4(message) => message
				.is_data_size_valid(num_of_parties, max_local_sigs_size(num_of_payloads.unwrap())),
		}
	}

	fn is_initial_stage_data_size_valid<Chain: ChainSigning>(&self) -> bool {
		match self {
			SigningData::CommStage1(message) => match Chain::CHAIN_TAG {
				ChainTag::Ethereum |
				ChainTag::Polkadot |
				ChainTag::Ed25519 |
				ChainTag::Arbitrum => message.payload.len() <= max_signing_commitments_size(1),
				ChainTag::Bitcoin =>
				// At this stage we may not know the number of payloads, so we use a maximum
					message.payload.len() <= max_signing_commitments_size(MAX_BTC_SIGNING_PAYLOADS),
			},
			_ => panic!("unexpected stage"),
		}
	}

	fn should_delay_unauthorised(&self) -> bool {
		matches!(self, SigningData::CommStage1(_))
	}

	fn should_delay(stage_name: SigningStageName, message: &Self) -> bool {
		match stage_name {
			SigningStageName::AwaitCommitments1 => {
				matches!(message, SigningData::BroadcastVerificationStage2(_))
			},
			SigningStageName::VerifyCommitmentsBroadcast2 => {
				matches!(message, SigningData::LocalSigStage3(_))
			},
			SigningStageName::LocalSigStage3 => {
				matches!(message, SigningData::VerifyLocalSigsStage4(_))
			},
			SigningStageName::VerifyLocalSigsBroadcastStage4 => {
				// Last stage, nothing to delay
				false
			},
		}
	}
}

#[cfg(test)]
mod tests {

	use crate::{
		bitcoin::BtcSigning,
		client::helpers::{gen_dummy_local_sig, gen_dummy_signing_comm1},
		crypto::eth::Point,
		eth::EthSigning,
		polkadot::PolkadotSigning,
		ChainSigning, Rng,
	};

	use rand::SeedableRng;

	use super::*;

	pub fn gen_signing_data_stage1(number_of_commitments: u64) -> SigningData<Point> {
		let mut rng = Rng::from_seed([0; 32]);
		SigningData::<Point>::CommStage1(gen_dummy_signing_comm1(&mut rng, number_of_commitments))
	}

	pub fn gen_signing_data_stage2(
		participant_count: AuthorityCount,
		number_of_commitments: usize,
	) -> SigningData<Point> {
		let mut rng = Rng::from_seed([0; 32]);
		SigningData::<Point>::BroadcastVerificationStage2(BroadcastVerificationMessage {
			data: (1..=participant_count)
				.map(|i| {
					(
						i as AuthorityCount,
						Some(gen_dummy_signing_comm1(&mut rng, number_of_commitments as u64)),
					)
				})
				.collect(),
		})
	}

	pub fn gen_signing_data_stage3(number_of_responses: usize) -> SigningData<Point> {
		let mut rng = Rng::from_seed([0; 32]);
		SigningData::<Point>::LocalSigStage3(gen_dummy_local_sig(
			&mut rng,
			number_of_responses as u64,
		))
	}

	pub fn gen_signing_data_stage4(
		participant_count: AuthorityCount,
		number_of_responses: usize,
	) -> SigningData<Point> {
		let mut rng = Rng::from_seed([0; 32]);
		SigningData::<Point>::VerifyLocalSigsStage4(BroadcastVerificationMessage {
			data: (1..=participant_count)
				.map(|i| {
					(
						i as AuthorityCount,
						Some(gen_dummy_local_sig(&mut rng, number_of_responses as u64)),
					)
				})
				.collect(),
		})
	}

	#[test]
	fn check_data_size_stage1() {
		// Should only pass if the message contains exactly one commitment for ethereum and Polkadot
		assert!(gen_signing_data_stage1(1).is_initial_stage_data_size_valid::<EthSigning>());
		assert!(!gen_signing_data_stage1(2).is_initial_stage_data_size_valid::<EthSigning>());
		assert!(!gen_signing_data_stage1(2).is_initial_stage_data_size_valid::<PolkadotSigning>());

		// Because we might not know the number of payloads yet, we limit btc to a constant
		assert!(gen_signing_data_stage1(MAX_BTC_SIGNING_PAYLOADS as u64)
			.is_initial_stage_data_size_valid::<BtcSigning>());
		assert!(gen_signing_data_stage1((MAX_BTC_SIGNING_PAYLOADS - 1) as u64)
			.is_initial_stage_data_size_valid::<BtcSigning>());
		assert!(!gen_signing_data_stage1((MAX_BTC_SIGNING_PAYLOADS + 1) as u64)
			.is_initial_stage_data_size_valid::<BtcSigning>());
	}

	#[test]
	fn check_data_size_stage2() {
		const PARTIES: AuthorityCount = 4;
		const PAYLOAD_COUNT: usize = 3;

		// Outer collection should fail on sizes larger or smaller than expected
		assert!(gen_signing_data_stage2(PARTIES, PAYLOAD_COUNT)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));
		assert!(!gen_signing_data_stage2(PARTIES - 1, PAYLOAD_COUNT)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));
		assert!(!gen_signing_data_stage2(PARTIES + 1, PAYLOAD_COUNT)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));

		// Inner collection should fail on sizes larger than the maximum size
		assert!(gen_signing_data_stage2(PARTIES, PAYLOAD_COUNT - 1)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));
		assert!(!gen_signing_data_stage2(PARTIES, PAYLOAD_COUNT + 1)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));
	}

	#[test]
	fn check_data_size_stage3() {
		const PARTIES: AuthorityCount = 4;
		const PAYLOAD_COUNT: usize = 3;

		// Should fail if it has too many responses
		assert!(gen_signing_data_stage3(PAYLOAD_COUNT)
			.is_data_size_valid::<EthSigning>(PARTIES, Some(PAYLOAD_COUNT)));
		assert!(gen_signing_data_stage3(PAYLOAD_COUNT - 1)
			.is_data_size_valid::<EthSigning>(PARTIES, Some(PAYLOAD_COUNT)));
		assert!(!gen_signing_data_stage3(PAYLOAD_COUNT + 1)
			.is_data_size_valid::<EthSigning>(PARTIES, Some(PAYLOAD_COUNT)));
	}

	#[test]
	fn check_data_size_stage4() {
		const PARTIES: AuthorityCount = 4;
		const PAYLOAD_COUNT: usize = 3;

		// Outer collection should fail on sizes larger or smaller than expected
		assert!(gen_signing_data_stage4(PARTIES, PAYLOAD_COUNT)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));
		assert!(!gen_signing_data_stage4(PARTIES - 1, PAYLOAD_COUNT)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));
		assert!(!gen_signing_data_stage4(PARTIES + 1, PAYLOAD_COUNT)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));

		// Inner collection should fail on sizes larger than the maximum size
		assert!(gen_signing_data_stage4(PARTIES, PAYLOAD_COUNT - 1)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));
		assert!(!gen_signing_data_stage4(PARTIES, PAYLOAD_COUNT + 1)
			.is_data_size_valid::<BtcSigning>(PARTIES, Some(PAYLOAD_COUNT)));
	}

	#[test]
	fn should_delay_correct_data_for_stage() {
		let mut rng = Rng::from_seed([0; 32]);
		let default_length = 1;

		let stage_name = [
			SigningStageName::AwaitCommitments1,
			SigningStageName::VerifyCommitmentsBroadcast2,
			SigningStageName::LocalSigStage3,
			SigningStageName::VerifyLocalSigsBroadcastStage4,
		];
		let stage_data = [
			gen_signing_data_stage1(default_length as u64),
			gen_signing_data_stage2(default_length, default_length as usize),
			SigningData::<Point>::LocalSigStage3(gen_dummy_local_sig(&mut rng, 1)),
			gen_signing_data_stage4(default_length, default_length as usize),
		];

		for (stage_index, name) in stage_name.iter().enumerate() {
			for (data_index, data) in stage_data.iter().enumerate() {
				if stage_index + 1 == data_index {
					// Should delay the next stage data (stage_index + 1)
					assert!(SigningData::should_delay(*name, data));
				} else {
					// Should not delay any other stage
					assert!(!SigningData::should_delay(*name, data));
				}
			}
		}
	}

	#[test]
	/// Check that each chain does not exceed an acceptable limit to the amount of ceremony data
	/// that a single node can force us to store as delayed initial stage messages.
	fn should_not_exceed_spam_limits() {
		const MULTI_PAYLOAD_SPAM_LIMIT_BYTES: u64 = 100_000_000; // ~100mb
		assert!(
			max_signing_commitments_size(MAX_BTC_SIGNING_PAYLOADS) as u64 *
				<BtcSigning as ChainSigning>::CEREMONY_ID_WINDOW <=
				MULTI_PAYLOAD_SPAM_LIMIT_BYTES
		);

		const SINGLE_PAYLOAD_SPAM_LIMIT_BYTES: u64 = 500_000; // ~0.5mb
		assert!(
			max_signing_commitments_size(1) as u64 *
				<EthSigning as ChainSigning>::CEREMONY_ID_WINDOW <=
				SINGLE_PAYLOAD_SPAM_LIMIT_BYTES
		);
		assert!(
			max_signing_commitments_size(1) as u64 *
				<PolkadotSigning as ChainSigning>::CEREMONY_ID_WINDOW <=
				SINGLE_PAYLOAD_SPAM_LIMIT_BYTES
		);
	}
}
