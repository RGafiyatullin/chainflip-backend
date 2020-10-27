//! Witness has the following responsibilities:
//! - It is subscribed to the side chain for *quote transactions*
//! - It monitors foreign blockchains for *incoming transactions*

// Events: Lokid transaction, Ether transaction, Swap transaction from Side Chain

mod btc;
mod btc_spv;
mod ethereum;
mod loki_witness;

pub use btc::BitcoinWitness;
pub use btc_spv::BtcSPVWitness;
pub use ethereum::EthereumWitness;
pub use loki_witness::LokiWitness;

pub mod fake_witness;
