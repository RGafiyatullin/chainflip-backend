use std::collections::{BTreeMap, BTreeSet};

use cf_traits::AuthorityCount;
use serde::{Deserialize, Serialize};

use crate::multisig::client::common::BroadcastVerificationMessage;

use super::keygen_frost::ShamirShare;

macro_rules! derive_impls_for_keygen_data {
    ($variant: ty, $variant_path: path) => {
        derive_impls_for_enum_variants!($variant, $variant_path, KeygenData);
    };
}

/// Data sent between parties over p2p for a keygen ceremony
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KeygenData {
    HashComm1(HashComm1),
    VerifyHashComm2(VerifyHashComm2),
    Comm1(Comm1),
    Verify2(VerifyComm2),
    SecretShares3(SecretShare3),
    Complaints4(Complaints4),
    VerifyComplaints5(VerifyComplaints5),
    BlameResponse6(BlameResponse6),
    VerifyBlameResponses7(VerifyBlameResponses7),
}

impl std::fmt::Display for KeygenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = match self {
            KeygenData::HashComm1(hash_comm1) => hash_comm1.to_string(),
            KeygenData::VerifyHashComm2(verify2) => verify2.to_string(),
            KeygenData::Comm1(comm1) => comm1.to_string(),
            KeygenData::Verify2(verify2) => verify2.to_string(),
            KeygenData::SecretShares3(share3) => share3.to_string(),
            KeygenData::Complaints4(complaints4) => complaints4.to_string(),
            KeygenData::VerifyComplaints5(verify5) => verify5.to_string(),
            KeygenData::BlameResponse6(blame_response) => blame_response.to_string(),
            KeygenData::VerifyBlameResponses7(verify7) => verify7.to_string(),
        };
        write!(f, "KeygenData({})", inner)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HashComm1(pub sp_core::H256);

pub type VerifyHashComm2 = BroadcastVerificationMessage<HashComm1>;

pub type Comm1 = super::keygen_frost::DKGUnverifiedCommitment;

pub type VerifyComm2 = BroadcastVerificationMessage<Comm1>;

/// Secret share of our locally generated secret calculated separately
/// for each party as the result of evaluating sharing polynomial (generated
/// during stage 1) at the corresponding signer's index
pub type SecretShare3 = ShamirShare;

/// List of parties blamed for sending invalid secret shares
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Complaints4(pub BTreeSet<AuthorityCount>);

pub type VerifyComplaints5 = BroadcastVerificationMessage<Complaints4>;

/// For each party blaming a node, it responds with the corresponding (valid)
/// secret share. Unlike secret shares sent at the earlier stage, these shares
/// are verifiably broadcast, so sending an invalid share would result in the
/// node being slashed. Although the shares are meant to be secret, it is safe
/// to reveal/broadcast some them at this stage: a node's long-term secret can
/// only be recovered by collecting shares from all (N-1) nodes, which would
/// require collusion of N-1 nodes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlameResponse6(pub BTreeMap<AuthorityCount, ShamirShare>);

pub type VerifyBlameResponses7 = BroadcastVerificationMessage<BlameResponse6>;

derive_impls_for_keygen_data!(HashComm1, KeygenData::HashComm1);
derive_impls_for_keygen_data!(VerifyHashComm2, KeygenData::VerifyHashComm2);
derive_impls_for_keygen_data!(Comm1, KeygenData::Comm1);
derive_impls_for_keygen_data!(VerifyComm2, KeygenData::Verify2);
derive_impls_for_keygen_data!(ShamirShare, KeygenData::SecretShares3);
derive_impls_for_keygen_data!(Complaints4, KeygenData::Complaints4);
derive_impls_for_keygen_data!(VerifyComplaints5, KeygenData::VerifyComplaints5);
derive_impls_for_keygen_data!(BlameResponse6, KeygenData::BlameResponse6);
derive_impls_for_keygen_data!(VerifyBlameResponses7, KeygenData::VerifyBlameResponses7);
