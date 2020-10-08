use crate::{common::store::KeyValueStore, vault::transactions::TransactionProvider};

pub use output::{CoinProcessor, LokiSender, OutputCoinProcessor};

/// Processing utils
pub mod utils;

/// Swap processing
mod swap;

/// Output processing
mod output;

/// Stake and unstake processing
mod staking;

/// Component that matches witness transactions with quotes and processes them
pub struct SideChainProcessor<T, KVS, S>
where
    T: TransactionProvider,
    KVS: KeyValueStore,
    S: CoinProcessor,
{
    tx_provider: T,
    db: KVS,
    coin_sender: S,
}

/// Events emited by the processor
#[derive(Debug)]
pub enum ProcessorEvent {
    /// Block id processed (including all earlier blocks)
    BLOCK(u32),
}

type EventSender = crossbeam_channel::Sender<ProcessorEvent>;

impl<T, KVS, S> SideChainProcessor<T, KVS, S>
where
    T: TransactionProvider + Send + Sync + 'static,
    KVS: KeyValueStore + Send + 'static,
    S: CoinProcessor + Send + 'static,
{
    /// Constructor taking a transaction provider
    pub fn new(tx_provider: T, kvs: KVS, coin_sender: S) -> Self {
        SideChainProcessor {
            tx_provider,
            db: kvs,
            coin_sender,
        }
    }

    async fn on_blockchain_progress(&mut self) {
        staking::process_stakes(&mut self.tx_provider);

        swap::process_swaps(&mut self.tx_provider);

        output::process_outputs(&mut self.tx_provider, &mut self.coin_sender).await;
    }

    /// Poll the side chain/tx_provider and use event_sender to
    /// notify of local events
    async fn run_event_loop(mut self, event_sender: Option<EventSender>) {
        const DB_KEY: &'static str = "processor_next_block_idx";

        // TODO: We should probably distinguish between no value and other errors here:
        // The first block that's yet to be processed by us
        let mut next_block_idx = self.db.get_data(DB_KEY).unwrap_or(0);

        info!("Processor starting with next block idx: {}", next_block_idx);

        loop {
            let idx = self.tx_provider.sync();

            debug!("Provider is at block: {}", idx);

            // Check if transaction provider made progress
            if idx >= next_block_idx {
                self.on_blockchain_progress().await;
            }

            if let Err(err) = self.db.set_data(DB_KEY, Some(idx)) {
                error!("Could not update latest block in db: {}", err);
                // Not quote sure how to recover from this, so probably best to terminate
                panic!("Database failure");
            }

            next_block_idx = idx;
            if let Some(sender) = &event_sender {
                let _ = sender.send(ProcessorEvent::BLOCK(idx));
                debug!("Processor processing block: {}", idx);
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        // Poll the side chain (via the transaction provider) and see if there are
        // any new witness transactions that should be processed
    }

    /// Start processor thread. If `event_sender` is provided,
    /// local events will be communicated through it.
    pub fn start(self, event_sender: Option<EventSender>) {
        std::thread::spawn(move || {
            info!("Starting the processor thread");

            let mut rt = tokio::runtime::Runtime::new().unwrap();

            rt.block_on(async {
                self.run_event_loop(event_sender).await;
            });
        });
    }
}
