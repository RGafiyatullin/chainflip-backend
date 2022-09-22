#![cfg(feature = "ibiza")]

use std::{collections::BTreeSet, pin::Pin, sync::Arc};

use async_trait::async_trait;
use cf_primitives::{Asset, EpochIndex, ForeignChainAddress};
use sp_core::H256;
use web3::{
    ethabi::{self, RawLog},
    types::H160,
};

use crate::state_chain_observer::client::{StateChainClient, StateChainRpcApi};

use super::{
    contract_witnesser::ContractStateUpdate, event::Event, rpc::EthRpcApi, utils, DecodeLogClosure,
    EthContractWitnesser, SignatureAndEvent,
};
use pallet_cf_ingress::IngressWitness;

// These are the two events that must be supported as part of the ERC20 standard
// https://eips.ethereum.org/EIPS/eip-20#events
#[derive(Debug)]
pub enum Erc20Event {
    Transfer {
        from: ethabi::Address,
        to: ethabi::Address,
        value: u128,
    },
    Approval {
        owner: ethabi::Address,
        spender: ethabi::Address,
        value: u128,
    },
    // A contract adhering to the ERC20 standard may also emit *more* than the standard events.
    // We don't care about these ones.
    Other(RawLog),
}

use anyhow::Result;

/// Can witness txs of a a particular ERC20 token to any of the monitored addresses.
/// NB: Any tokens watched by this must *strictly* adhere to the ERC20 standard: https://eips.ethereum.org/EIPS/eip-20
pub struct Erc20Witnesser {
    pub deployed_address: H160,
    asset: Asset,
    contract: ethabi::Contract,
}

impl Erc20Witnesser {
    /// Loads the contract abi to get the event definitions
    pub fn new(deployed_address: H160, asset: Asset) -> Self {
        Self {
            deployed_address,
            asset,
            contract: ethabi::Contract::load(std::include_bytes!("abis/ERC20.json").as_ref())
                .unwrap(),
        }
    }
}

pub struct Erc20WitnesserState {
    monitored_addresses: BTreeSet<H160>,
    address_receiver: tokio::sync::mpsc::UnboundedReceiver<H160>,
}

impl Erc20WitnesserState {
    pub fn new(
        monitored_addresses: BTreeSet<H160>,
        address_receiver: tokio::sync::mpsc::UnboundedReceiver<H160>,
    ) -> Self {
        Self {
            monitored_addresses,
            address_receiver,
        }
    }
}

impl ContractStateUpdate for Erc20WitnesserState {
    type Item = H160;

    fn next_item_to_update(
        &mut self,
    ) -> Pin<Box<dyn futures::Future<Output = Option<Self::Item>> + Send + '_>> {
        Box::pin(self.address_receiver.recv())
    }

    fn update_state(&mut self, new_address: Self::Item) {
        self.monitored_addresses.insert(new_address);
    }

    fn should_act_on(&self, item: &Self::Item) -> bool {
        self.monitored_addresses.contains(item)
    }
}

#[async_trait]
impl EthContractWitnesser for Erc20Witnesser {
    type EventParameters = Erc20Event;
    type StateItem = H160;

    // TODO: Include asset in name
    fn contract_name(&self) -> &'static str {
        "ERC20"
    }

    async fn handle_event<RpcClient, EthRpcClient, ContractWitnesserState>(
        &self,
        epoch: EpochIndex,
        _block_number: u64,
        event: Event<Self::EventParameters>,
        filter_state: &ContractWitnesserState,
        state_chain_client: Arc<StateChainClient<RpcClient>>,
        _eth_rpc: &EthRpcClient,
        logger: &slog::Logger,
    ) -> Result<()>
    where
        RpcClient: 'static + StateChainRpcApi + Sync + Send,
        EthRpcClient: EthRpcApi + Sync + Send,
        ContractWitnesserState: Send + Sync + ContractStateUpdate<Item = Self::StateItem>,
    {
        if let Erc20Event::Transfer { to, value, from: _ } = event.event_parameters {
            if filter_state.should_act_on(&to) {
                let _result = state_chain_client
                    .submit_signed_extrinsic(
                        pallet_cf_witnesser::Call::witness_at_epoch {
                            call: Box::new(
                                pallet_cf_ingress::Call::do_ingress {
                                    ingress_witnesses: vec![IngressWitness {
                                        ingress_address: ForeignChainAddress::Eth(to.into()),
                                        amount: value,
                                        asset: self.asset,
                                        tx_hash: event.tx_hash,
                                    }],
                                }
                                .into(),
                            ),
                            epoch_index: epoch,
                        },
                        logger,
                    )
                    .await;
            }
        }
        Ok(())
    }

    fn get_contract_address(&self) -> H160 {
        self.deployed_address
    }

    fn decode_log_closure(&self) -> Result<DecodeLogClosure<Self::EventParameters>> {
        let transfer = SignatureAndEvent::new(&self.contract, "Transfer")?;
        let approval = SignatureAndEvent::new(&self.contract, "Approval")?;

        Ok(Box::new(
            move |event_signature: H256, raw_log: RawLog| -> Result<Self::EventParameters> {
                Ok(if event_signature == transfer.signature {
                    let log = transfer.event.parse_log(raw_log)?;
                    Erc20Event::Transfer {
                        from: utils::decode_log_param(&log, "from")?,
                        to: utils::decode_log_param(&log, "to")?,
                        value: utils::decode_log_param::<ethabi::Uint>(&log, "value")?.as_u128(),
                    }
                } else if event_signature == approval.signature {
                    let log = approval.event.parse_log(raw_log)?;
                    Erc20Event::Approval {
                        owner: utils::decode_log_param(&log, "owner")?,
                        spender: utils::decode_log_param(&log, "spender")?,
                        value: utils::decode_log_param::<ethabi::Uint>(&log, "value")?.as_u128(),
                    }
                } else {
                    Erc20Event::Other(raw_log)
                })
            },
        ))
    }
}

// Convenience test to allow us to generate the signatures of the events, allowing us
// to manually query the contract for the events
// current signatures below:
// transfer: 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
// approval: 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
#[test]
fn generate_signatures() {
    let contract = Erc20Witnesser::new(H160::default(), Asset::Flip).contract;

    let transfer = SignatureAndEvent::new(&contract, "Transfer").unwrap();
    println!("transfer: {:?}", transfer.signature);
    let approval = SignatureAndEvent::new(&contract, "Approval").unwrap();
    println!("approval: {:?}", approval.signature);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_load_contract() {
        let address = H160::default();
        Erc20Witnesser::new(address, Asset::Flip);
    }

    #[test]
    fn test_transfer_log_parsing() {
        let erc20_witnesser = Erc20Witnesser::new(H160::default(), Asset::Flip);
        let decode_log = erc20_witnesser.decode_log_closure().unwrap();

        let transfer_event_signature =
            H256::from_str("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")
                .unwrap();

        // RawLog taken from event on FLIP contract (which adheres to ERC20 standard)
        match decode_log(
            transfer_event_signature,
            RawLog {
                topics: vec![
                    transfer_event_signature,
                    H256::from_str(
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                    )
                    .unwrap(),
                    H256::from_str(
                        "0x0000000000000000000000009fe46736679d2d9a65f0992f2272de9f3c7fa6e0",
                    )
                    .unwrap(),
                ],
                data: hex::decode(
                    "0000000000000000000000000000000000000000000034f086f3b33b68400000",
                )
                .unwrap(),
            },
        )
        .unwrap()
        {
            Erc20Event::Transfer { from, to, value } => {
                assert_eq!(
                    from,
                    web3::types::H160::from_str("0x0000000000000000000000000000000000000000")
                        .unwrap(),
                    "from address not matching"
                );
                assert_eq!(
                    to,
                    web3::types::H160::from_str("0x9fe46736679d2d9a65f0992f2272de9f3c7fa6e0")
                        .unwrap(),
                    "to address not matching"
                );
                assert_eq!(value, 250000000000000000000000u128, "value not matching");
            }
            _ => panic!("Expected Erc20Eevent::Transfer, got a different variant"),
        }
    }

    #[test]
    fn test_approval_log_parsing() {
        let erc20_witnesser = Erc20Witnesser::new(H160::default(), Asset::Flip);
        let decode_log = erc20_witnesser.decode_log_closure().unwrap();

        let approval_event_signature =
            H256::from_str("0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925")
                .unwrap();

        // RawLog taken from event on FLIP contract (which adheres to ERC20 standard)
        match decode_log(
            approval_event_signature,
            RawLog {
                topics: vec![
                    approval_event_signature,
                    H256::from_str(
                        "0x00000000000000000000000070997970c51812dc3a010c7d01b50e0d17dc79c8",
                    )
                    .unwrap(),
                    H256::from_str(
                        "0x0000000000000000000000009fe46736679d2d9a65f0992f2272de9f3c7fa6e0",
                    )
                    .unwrap(),
                ],
                data: hex::decode(
                    "000000000000000000000000000000000000000000084595161401484a000000",
                )
                .unwrap(),
            },
        )
        .unwrap()
        {
            Erc20Event::Approval {
                owner,
                spender,
                value,
            } => {
                assert_eq!(
                    owner,
                    web3::types::H160::from_str("0x70997970c51812dc3a010c7d01b50e0d17dc79c8")
                        .unwrap(),
                    "owner address not matching"
                );
                assert_eq!(
                    spender,
                    web3::types::H160::from_str("0x9fe46736679d2d9a65f0992f2272de9f3c7fa6e0")
                        .unwrap(),
                    "spender address not matching"
                );
                assert_eq!(value, 10000000000000000000000000u128, "value not matching");
            }
            _ => panic!("Expected Erc20Event::Approval, got a different variant"),
        }
    }
}
