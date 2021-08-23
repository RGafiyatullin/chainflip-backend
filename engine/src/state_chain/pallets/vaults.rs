use std::marker::PhantomData;

use codec::{Decode, Encode};
use pallet_cf_vaults::KeygenRequest;
use sp_runtime::AccountId32;
use substrate_subxt::{module, system::System, Event};

use serde::{Deserialize, Serialize};

#[module]
pub trait Vaults: System {}

// The order of these fields matter for decoding
#[derive(Clone, Debug, Eq, PartialEq, Event, Encode, Decode)]
pub struct KeygenRequestEvent<V: Vaults> {
    pub request_index: u64,

    pub keygen_request: KeygenRequest<AccountId32>,

    pub _runtime: PhantomData<V>,
}

// // The order of these fields matter for decoding
// #[derive(Clone, Debug, Eq, PartialEq, Event, Encode, Decode, Serialize, Deserialize)]
// pub struct VaultRotationRequestEvent<V: Vaults> {
//     pub _runtime: PhantomData<V>,
// }

// // The order of these fields matter for decoding
// #[derive(Clone, Debug, Eq, PartialEq, Event, Encode, Decode, Serialize, Deserialize)]
// pub struct VaultRotationCompletedEvent<V: Vaults> {
//     pub _runtime: PhantomData<V>,
// }

// // The order of these fields matter for decoding
// #[derive(Clone, Debug, Eq, PartialEq, Event, Encode, Decode, Serialize, Deserialize)]
// pub struct RotationAbortedEvent<V: Vaults> {
//     pub _runtime: PhantomData<V>,
// }

// // The order of these fields matter for decoding
// #[derive(Clone, Debug, Eq, PartialEq, Event, Encode, Decode, Serialize, Deserialize)]
// pub struct VaultsRotatedEvent<V: Vaults> {
//     pub _runtime: PhantomData<V>,
// }

// // The order of these fields matter for decoding
// #[derive(Clone, Debug, Eq, PartialEq, Event, Encode, Decode, Serialize, Deserialize)]
// pub struct EthSignTxRequestEvent<V: Vaults> {
//     pub _runtime: PhantomData<V>,
// }
