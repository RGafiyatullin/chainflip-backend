use super::vault_node::VaultNodeInterface;
use super::BlockProcessor;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::{thread, time};

/// A struct which can poll for blocks
pub struct BlockPoller<V, P>
where
    V: VaultNodeInterface + Send + Sync,
    P: BlockProcessor + Send + Sync,
{
    api: Arc<V>,
    processor: Arc<P>,
    next_block_number: AtomicU32,
}

impl<V, P> BlockPoller<V, P>
where
    V: VaultNodeInterface + Send + Sync + 'static,
    P: BlockProcessor + Send + Sync + 'static,
{
    /// Create a new block poller
    pub fn new(api: Arc<V>, processor: Arc<P>) -> Self {
        let last_block_number = processor.get_last_processed_block_number();
        let next_block_number = if let Some(number) = last_block_number {
            number + 1
        } else {
            0
        };

        BlockPoller {
            api,
            processor,
            next_block_number: AtomicU32::new(next_block_number),
        }
    }

    /// Poll until we have reached the latest block.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Any error occurs while trying to fetch blocks from the api after 3 retries.
    /// * Any error occurs while processing blocks.
    ///
    /// # Panics
    ///
    /// Panics if we detected any skipped blocks.
    /// This can happen if `VaultNodeInterface::get_blocks` returns partial data.
    pub fn sync(&self) -> Result<(), String> {
        let mut error_count = 0;
        loop {
            let next_block_number = self.next_block_number.load(Ordering::SeqCst);
            match self.api.get_blocks(next_block_number, 50) {
                Ok(blocks) => {
                    if blocks.is_empty() {
                        return Ok(());
                    }

                    let last_block_number = blocks.iter().map(|b| b.number).max();

                    // Validate the returned block numbers to make sure we didn't skip
                    let expected_last_block_number = next_block_number + (blocks.len() as u32) - 1; // assumption: getBlock(2, 4) will get us blocks 2,3,4,5
                    if let Some(last_block_number) = last_block_number {
                        if last_block_number != expected_last_block_number {
                            error!("Expected last block number to be {} but got {}. We must've skipped block!", last_block_number, expected_last_block_number);
                            panic!("BlockPoller skipped blocks!");
                        }
                    }

                    // Pass blocks off to processor
                    self.processor.process_blocks(blocks)?;

                    // Update our local value
                    if let Some(last_block_number) = last_block_number {
                        if last_block_number + 1 > next_block_number {
                            self.next_block_number
                                .store(last_block_number + 1, Ordering::SeqCst);
                        }
                    }

                    error_count = 0;
                }
                Err(e) => {
                    if error_count > 3 {
                        return Err(e);
                    } else {
                        error_count += 1
                    }
                }
            }
        }
    }

    /// Start polling indefinately.
    /// This will consume the BlockPoller.
    pub fn poll(self) -> thread::JoinHandle<()> {
        return thread::spawn(move || loop {
            if let Err(e) = self.sync() {
                info!("Block Poller ran into an error while polling: {}", e);
            }

            // Wait for a while before fetching again
            thread::sleep(time::Duration::from_secs(1));
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::side_chain::SideChainBlock;
    use crate::utils::test_utils::block_processor::TestBlockProcessor;
    use crate::utils::test_utils::vault_node_api::TestVaultNodeAPI;

    struct TestVariables {
        poller: BlockPoller<TestVaultNodeAPI, TestBlockProcessor>,
        api: Arc<TestVaultNodeAPI>,
        processor: Arc<TestBlockProcessor>,
    }

    fn setup() -> TestVariables {
        let api = Arc::new(TestVaultNodeAPI::new());
        let processor = Arc::new(TestBlockProcessor::new());
        let poller = BlockPoller::new(api.clone(), processor.clone());

        return TestVariables {
            poller,
            api,
            processor,
        };
    }

    #[test]
    fn test_sync_returns_when_no_blocks_returned() {
        let state = setup();
        assert!(state.api.get_blocks_return.lock().unwrap().is_empty());
        assert!(state.poller.sync().is_ok());
    }

    #[test]
    fn test_sync_returns_error_if_api_failed() {
        let state = setup();
        state
            .api
            .set_get_blocks_error(Some(String::from("TestError")));
        assert_eq!(state.poller.sync().unwrap_err(), String::from("TestError"));
    }

    #[test]
    #[should_panic(expected = "BlockPoller skipped blocks!")]
    fn test_sync_panics_when_blocks_are_skipped() {
        let state = setup();
        state.api.add_blocks(vec![
            SideChainBlock {
                number: 1,
                txs: vec![],
            },
            SideChainBlock {
                number: 100,
                txs: vec![],
            },
        ]);
        state.poller.next_block_number.store(1, Ordering::SeqCst);
        state.poller.sync().unwrap();
    }

    #[test]
    fn test_sync_updates_next_block_number_only_if_larger() -> Result<(), String> {
        let state = setup();
        state.api.add_blocks(vec![
            SideChainBlock {
                number: 0,
                txs: vec![],
            },
            SideChainBlock {
                number: 1,
                txs: vec![],
            },
        ]);

        state.poller.sync()?;
        assert_eq!(state.poller.next_block_number.load(Ordering::SeqCst), 2);
        return Ok(());
    }

    #[test]
    fn test_sync_loops_through_all_blocks() -> Result<(), String> {
        let state = setup();
        state.api.add_blocks(vec![
            SideChainBlock {
                number: 0,
                txs: vec![],
            },
            SideChainBlock {
                number: 1,
                txs: vec![],
            },
        ]);
        state.api.add_blocks(vec![SideChainBlock {
            number: 2,
            txs: vec![],
        }]);

        state.poller.sync()?;
        assert_eq!(state.poller.next_block_number.load(Ordering::SeqCst), 3);
        assert_eq!(state.processor.recieved_blocks.lock().unwrap().len(), 3);
        return Ok(());
    }

    #[test]
    fn test_sync_passes_blocks_to_processor() -> Result<(), String> {
        let state = setup();
        state.api.add_blocks(vec![SideChainBlock {
            number: 0,
            txs: vec![],
        }]);
        state.poller.sync()?;
        assert_eq!(state.processor.recieved_blocks.lock().unwrap().len(), 1);
        assert_eq!(
            state
                .processor
                .recieved_blocks
                .lock()
                .unwrap()
                .get(0)
                .unwrap()
                .number,
            0
        );
        return Ok(());
    }

    #[test]
    fn test_sync_returns_error_if_processor_failed() {
        let error = String::from("ProcessorTestError");
        let state = setup();

        state.api.add_blocks(vec![SideChainBlock {
            number: 0,
            txs: vec![],
        }]);
        state
            .processor
            .set_process_blocks_error(Some(error.clone()));

        assert_eq!(state.poller.sync().unwrap_err(), error);
    }
}
