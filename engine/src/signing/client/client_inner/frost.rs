//! The types and operations as discussed in <https://eprint.iacr.org/2020/852.pdf>.
//! Comments in this file reference sections from this document.
//! Note that unlike the protocol described in the document, we don't have a
//! centralised signature aggregator and don't have a preprocessing stage.

use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt::Display,
};

use pallet_cf_vaults::CeremonyId;
use serde::{Deserialize, Serialize};

use super::{
    client_inner::MultisigMessage, common::BroadcastVerificationMessage, SchnorrSignature,
};
use pallet_cf_vaults::crypto::destructure_pubkey;

use crate::signing::crypto::{BigInt, BigIntConverter, ECPoint, ECScalar, KeyShare, Point, Scalar};

use sha2::{Digest, Sha256};

/// A pair of secret single-use nonces (and their
/// corresponding public commitments). Correspond to (d,e)
/// generated during the preprocessing stage in Secion 5.3 (page 13)
// TODO: Not sure if it is a good idea to to make
// the secret values clonable
#[derive(Clone)]
pub struct SecretNoncePair {
    pub d: Scalar,
    pub d_pub: Point,
    pub e: Scalar,
    pub e_pub: Point,
}

impl SecretNoncePair {
    /// Generate a random pair of nonces
    pub fn sample_random() -> Self {
        let d = Scalar::new_random();
        let e = Scalar::new_random();

        let d_pub = Point::generator() * d;
        let e_pub = Point::generator() * e;

        SecretNoncePair { d, d_pub, e, e_pub }
    }
}

/// Public components of the single-use nonces generated by
/// a single party at signer index `index`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub struct SigningCommitment {
    pub index: usize,
    pub d: Point,
    pub e: Point,
}

pub type Comm1 = SigningCommitment;

pub type VerifyComm2 = BroadcastVerificationMessage<Comm1>;
pub type VerifyLocalSig4 = BroadcastVerificationMessage<LocalSig3>;

/// Signature (the "response" part) shard generated by a single party
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalSig3 {
    pub response: Scalar,
}

macro_rules! derive_from_enum {
    ($variant: ty, $variant_path: path, $enum: ty) => {
        impl From<$variant> for $enum {
            fn from(x: $variant) -> Self {
                $variant_path(x)
            }
        }
    };
}

macro_rules! derive_try_from_variant {
    ($variant: ty, $variant_path: path, $enum: ty) => {
        impl std::convert::TryFrom<$enum> for $variant {
            type Error = &'static str;

            fn try_from(data: $enum) -> Result<Self, Self::Error> {
                if let $variant_path(x) = data {
                    Ok(x)
                } else {
                    Err(stringify!($enum))
                }
            }
        }
    };
}

macro_rules! derive_impls_for_enum_variants {
    ($variant: ty, $variant_path: path, $enum: ty) => {
        derive_from_enum!($variant, $variant_path, $enum);
        derive_try_from_variant!($variant, $variant_path, $enum);
        derive_display_as_type_name!($variant);
    };
}

macro_rules! derive_impls_for_signing_data {
    ($variant: ty, $variant_path: path) => {
        derive_impls_for_enum_variants!($variant, $variant_path, SigningData);
    };
}

/// Data exchanged between parties during various stages
/// of the FROST signing protocol
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SigningData {
    CommStage1(Comm1),
    BroadcastVerificationStage2(VerifyComm2),
    LocalSigStage3(LocalSig3),
    VerifyLocalSigsStage4(VerifyLocalSig4),
}

derive_impls_for_signing_data!(Comm1, SigningData::CommStage1);
derive_impls_for_signing_data!(VerifyComm2, SigningData::BroadcastVerificationStage2);
derive_impls_for_signing_data!(LocalSig3, SigningData::LocalSigStage3);
derive_impls_for_signing_data!(VerifyLocalSig4, SigningData::VerifyLocalSigsStage4);

impl Display for SigningData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = match self {
            SigningData::CommStage1(x) => x.to_string(),
            SigningData::BroadcastVerificationStage2(x) => x.to_string(),
            SigningData::LocalSigStage3(x) => x.to_string(),
            SigningData::VerifyLocalSigsStage4(x) => x.to_string(),
        };
        write!(f, "SigningData({})", inner)
    }
}

/// Helps identify data as used for
/// a specific signing ceremony
#[derive(Serialize, Deserialize, Debug)]
pub struct SigningDataWrapped {
    pub data: SigningData,
    pub ceremony_id: CeremonyId,
}

impl SigningDataWrapped {
    pub fn new<S>(data: S, ceremony_id: CeremonyId) -> Self
    where
        S: Into<SigningData>,
    {
        SigningDataWrapped {
            data: data.into(),
            ceremony_id,
        }
    }
}

impl From<SigningDataWrapped> for MultisigMessage {
    fn from(wrapped: SigningDataWrapped) -> Self {
        MultisigMessage::SigningMessage(wrapped)
    }
}

/// Combine individual commitments into group (schnorr) commitment.
/// See "Signing Protocol" in Section 5.2 (page 14).
fn gen_group_commitment(
    signing_commitments: &[SigningCommitment],
    bindings: &HashMap<usize, Scalar>,
) -> Point {
    signing_commitments
        .iter()
        .map(|comm| {
            let rho_i = bindings[&comm.index];
            comm.d + comm.e * rho_i
        })
        .reduce(|a, b| a + b)
        .expect("non empty list")
}

/// Generate a lagrange coefficient for party `signer_index`
/// according to Section 4 (page 9)
pub fn get_lagrange_coeff(
    signer_index: usize,
    all_signer_indices: &[usize],
) -> Result<Scalar, &'static str> {
    let mut num: Scalar = ECScalar::from(&BigInt::from(1));
    let mut den: Scalar = ECScalar::from(&BigInt::from(1));

    for j in all_signer_indices {
        if *j == signer_index {
            continue;
        }
        let j: Scalar = ECScalar::from(&BigInt::from(*j as u32));
        let signer_index: Scalar = ECScalar::from(&BigInt::from(signer_index as u32));
        num = num * j;
        den = den * (j.sub(&signer_index.get_element()));
    }

    if den == Scalar::zero() {
        return Err("Duplicate shares provided");
    }

    let lagrange_coeff = num * den.invert();

    Ok(lagrange_coeff)
}

/// Generate a "binding value" for party `index`. See "Signing Protocol" in Section 5.2 (page 14)
fn gen_rho_i(index: usize, msg: &[u8], signing_commitments: &[SigningCommitment]) -> Scalar {
    let mut hasher = Sha256::new();
    hasher.update(b"I");
    hasher.update(index.to_be_bytes());
    hasher.update(msg);

    for com in signing_commitments {
        hasher.update(com.index.to_be_bytes());
        hasher.update(com.d.get_element().serialize());
        hasher.update(com.e.get_element().serialize());
    }

    let result = hasher.finalize();

    let x: [u8; 32] = result.as_slice().try_into().expect("Invalid hash size");

    let x_bi = BigInt::from_bytes(&x);

    ECScalar::from(&x_bi)
}

type SigningResponse = LocalSig3;

/// Generate binding values for each party given their previously broadcast commitments
fn generate_bindings(msg: &[u8], commitments: &[SigningCommitment]) -> HashMap<usize, Scalar> {
    commitments
        .iter()
        .map(|c| (c.index, gen_rho_i(c.index, msg, commitments)))
        .collect()
}

/// Generate local signature/response (shard). See step 5 in Figure 3 (page 15).
pub fn generate_local_sig(
    msg: &[u8],
    key: &KeyShare,
    nonces: &SecretNoncePair,
    commitments: &[SigningCommitment],
    own_idx: usize,
    all_idxs: &[usize],
) -> SigningResponse {
    let bindings = generate_bindings(&msg, commitments);

    // This is `R` in a Schnorr signature
    let group_commitment = gen_group_commitment(&commitments, &bindings);

    let SecretNoncePair { d, e, .. } = nonces;

    let lambda_i = get_lagrange_coeff(own_idx, all_idxs).expect("lagrange coeff");

    let rho_i = bindings[&own_idx];

    let nonce_share = *d + (*e * rho_i);

    let key_share = lambda_i * key.x_i;

    let response =
        generate_contract_schnorr_sig(key_share, key.y, group_commitment, nonce_share, msg);

    SigningResponse { response }
}

/// Schnorr signature as defined by the Key Manager contract
fn generate_contract_schnorr_sig(
    private_key: Scalar,
    pubkey: Point,
    nonce_commitment: Point,
    nonce: Scalar,
    message: &[u8],
) -> Scalar {
    let challenge = build_challenge(
        pubkey.get_element(),
        nonce_commitment.get_element(),
        message,
    );

    nonce.sub(&(private_key * challenge).get_element())
}

/// Check the validity of a signature response share.
/// (See step 7.b in Figure 3, page 15.)
fn is_party_resonse_valid(
    y_i: &Point,
    lambda_i: &Scalar,
    commitment: &Point,
    challenge: &Scalar,
    signature_response: &Scalar,
) -> bool {
    (Point::generator() * signature_response)
        == (commitment.sub_point(&(y_i * challenge * lambda_i).get_element()))
}

/// Combine local signatures received from all parties into the final
/// (aggregate) signature given that no party misbehavied. Otherwise
/// return the misbehaving parties.
pub fn aggregate_signature(
    msg: &[u8],
    signer_idxs: &[usize],
    agg_pubkey: Point,
    pubkeys: &[Point],
    commitments: &[SigningCommitment],
    responses: &[SigningResponse],
) -> Result<SchnorrSignature, Vec<usize>> {
    let bindings = generate_bindings(&msg, commitments);

    let group_commitment = gen_group_commitment(commitments, &bindings);

    let challenge = build_challenge(
        agg_pubkey.get_element(),
        group_commitment.get_element(),
        msg,
    );

    let mut invalid_idxs = vec![];

    for signer_idx in signer_idxs {
        let array_index = signer_idx - 1;

        let rho_i = bindings[&signer_idx];
        let lambda_i = get_lagrange_coeff(*signer_idx, signer_idxs).unwrap();

        let commitment = &commitments[array_index];
        let commitment_i = commitment.d + (commitment.e * rho_i);

        let y_i = pubkeys[array_index];

        let response = &responses[array_index];

        if !is_party_resonse_valid(
            &y_i,
            &lambda_i,
            &commitment_i,
            &challenge,
            &response.response,
        ) {
            invalid_idxs.push(*signer_idx);
        }
    }

    if invalid_idxs.is_empty() {
        // Response shares/shards are additive, so we simply need to
        // add them together (see step 7.c in Figure 3, page 15).
        let z = responses
            .iter()
            .fold(Scalar::zero(), |acc, x| acc + x.response);

        Ok(SchnorrSignature {
            s: *z.get_element().as_ref(),
            r: group_commitment.get_element(),
        })
    } else {
        Err(invalid_idxs)
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use super::*;

    const SECRET_KEY: &str = "fbcb47bc85b881e0dfb31c872d4e06848f80530ccbd18fc016a27c4a744d0eba";
    const NONCE_KEY: &str = "d51e13c68bf56155a83e50fd9bc840e2a1847fb9b49cd206a577ecd1cd15e285";
    const MESSAGE_HASH: &str = "2bdc19071c7994f088103dbf8d5476d6deb6d55ee005a2f510dc7640055cc84e";

    // Through integration tests with the KeyManager contract we know this
    // to be deemed valid by the contract for the data above
    const EXPECTED_SIGMA: &str = "beb37e87509e15cd88b19fa224441c56acc0e143cb25b9fd1e57fdafed215538";

    fn scalar_from_secretkey(secret_key: secp256k1::SecretKey) -> Scalar {
        let mut scalar = Scalar::new_random();
        scalar.set_element(secret_key);
        scalar
    }

    fn scalar_from_secretkey_hex(secret_key_hex: &str) -> Scalar {
        let sk = secp256k1::SecretKey::from_str(secret_key_hex).expect("invalid hex");
        scalar_from_secretkey(sk)
    }

    #[test]
    fn signature_is_contract_compatible() {
        // Given the signing key, nonce and message hash, check that
        // sigma (signature response) is correct and matches the expected
        // (by the KeyManager contract) value
        let message = hex::decode(MESSAGE_HASH).unwrap();

        let nonce = scalar_from_secretkey_hex(NONCE_KEY);
        let commitment = Point::generator() * nonce;

        let private_key = scalar_from_secretkey_hex(SECRET_KEY);
        let public_key = Point::generator() * private_key;

        let response =
            generate_contract_schnorr_sig(private_key, public_key, commitment, nonce, &message);

        assert_eq!(
            hex::encode(response.get_element().as_ref()),
            "beb37e87509e15cd88b19fa224441c56acc0e143cb25b9fd1e57fdafed215538"
        );

        // Build the challenge again to match how it is done on the receiving side
        let challenge =
            build_challenge(public_key.get_element(), commitment.get_element(), &message);

        // A lambda that has no effect on the computation (as a way to adapt multi-party
        // signing to work for a single party)
        let dummy_lambda = ECScalar::from(&BigInt::from(1));

        assert!(is_party_resonse_valid(
            &public_key,
            &dummy_lambda,
            &commitment,
            &challenge,
            &response,
        ));
    }
}

/// Assembles and hashes the challenge in the correct order for the KeyManager Contract
fn build_challenge(
    pubkey: secp256k1::PublicKey,
    nonce_commitment: secp256k1::PublicKey,
    message: &[u8],
) -> Scalar {
    use sp_core::Hasher;
    use sp_runtime::traits::Keccak256;

    let eth_addr = crate::eth::utils::pubkey_to_eth_addr(nonce_commitment);

    let (pubkey_x, pubkey_y_parity) =
        destructure_pubkey(pubkey.serialize().to_vec()).expect("Should be valid pubkey");

    // Assemble the challenge in correct order according to this contract:
    // https://github.com/chainflip-io/chainflip-eth-contracts/blob/master/contracts/abstract/SchnorrSECP256K1.sol
    let e_bytes: Vec<_> = pubkey_x
        .iter()
        .chain([pubkey_y_parity].iter())
        .chain(message)
        .chain(eth_addr.iter())
        .cloned()
        .collect();

    let e_bn = BigInt::from_bytes(Keccak256::hash(&e_bytes).as_bytes());
    ECScalar::from(&e_bn)
}
