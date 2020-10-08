use crate::{
    common::{Liquidity, PoolCoin},
    side_chain::SideChainBlock,
    transactions::{OutputSentTx, OutputTx, QuoteTx, StakeQuoteTx, WitnessTx},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use vault_node::VaultNodeInterface;

mod api;
mod block_poller;

use api::API;

/// The quoter database
pub mod database;

/// The vault node api consumer
pub mod vault_node;

/// The config
pub mod config;

/// Quoter
pub struct Quoter {}

impl Quoter {
    /// Run the Quoter logic.
    ///
    /// # Blocking
    ///
    /// This will block the thread it is run on.
    pub async fn run<V, D>(
        port: u16,
        vault_node_api: Arc<V>,
        database: Arc<Mutex<D>>,
    ) -> Result<(), String>
    where
        V: VaultNodeInterface + Send + Sync + 'static,
        D: BlockProcessor + StateProvider + Send + 'static,
    {
        let poller = block_poller::BlockPoller::new(vault_node_api.clone(), database.clone());
        poller.sync()?; // Make sure we have all the latest blocks

        // Start loops
        let poller_thread = std::thread::spawn(move || {
            poller.poll(std::time::Duration::from_secs(1));
        });

        API::serve(port, vault_node_api.clone(), database.clone());

        poller_thread
            .join()
            .map_err(|_| "An error occurred while polling".to_owned())?;

        Ok(())
    }
}

/// A trait for processing side chain blocks received from the vault node.
pub trait BlockProcessor {
    /// Get the block number that was last processed.
    fn get_last_processed_block_number(&self) -> Option<u32>;

    /// Process a list of blocks
    fn process_blocks(&mut self, blocks: &[SideChainBlock]) -> Result<(), String>;
}

/// A trait for providing quoter state
pub trait StateProvider {
    /// Get all swap quotes
    fn get_swap_quotes(&self) -> Option<Vec<QuoteTx>>;
    /// Get swap quote with the given id
    fn get_swap_quote_tx(&self, id: String) -> Option<QuoteTx>;
    /// Get all stake quotes
    fn get_stake_quotes(&self) -> Option<Vec<StakeQuoteTx>>;
    /// Get stake quore with the given id
    fn get_stake_quote_tx(&self, id: String) -> Option<StakeQuoteTx>;
    /// Get all witness transactions with the given quote id
    fn get_witness_txs(&self, quote_id: String) -> Option<Vec<WitnessTx>>;
    /// Get all output transactions with the given quote id
    fn get_output_txs(&self, quote_id: String) -> Option<Vec<OutputTx>>;
    /// Get all output sent transactions
    fn get_output_sent_txs(&self) -> Option<Vec<OutputSentTx>>;
    /// Get the pools
    fn get_pools(&self) -> HashMap<PoolCoin, Liquidity>;
}
