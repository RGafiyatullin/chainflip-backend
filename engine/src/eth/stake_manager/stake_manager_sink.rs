use crate::{
    eth::EventSink,
    mq::mq::{self, IMQClient, Subject},
};

use async_trait::async_trait;

use super::stake_manager::StakingEvent;

use anyhow::Result;

/// A sink that can be used with an EthEventStreamer instance
/// Pushes events to the message queue
pub struct StakeManagerSink<M: IMQClient + Send + Sync> {
    mq_client: M,
}

impl<M: IMQClient + Send + Sync> StakeManagerSink<M> {
    pub async fn new(mq_options: mq::Options) -> Result<Self> {
        let mq_client = *M::connect(mq_options).await?;

        Ok(StakeManagerSink { mq_client })
    }
}

#[async_trait]
impl<M: IMQClient + Send + Sync> EventSink<StakingEvent> for StakeManagerSink<M> {
    async fn process_event(&self, event: StakingEvent) -> anyhow::Result<()> {
        log::trace!("Processing event in StakeManagerSink: {:?}", event);
        self.mq_client
            .publish(Subject::StakeManager, &event)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::mq::{mq::Options, nats_client::NatsMQClient};

    use super::*;

    #[tokio::test]
    // Ensure it doesn't panic
    async fn create_stake_manager_sink() {
        let server = nats_test_server::NatsTestServer::build().spawn();
        let addr = server.address().to_string();
        let options = Options { url: addr };
        StakeManagerSink::<NatsMQClient>::new(options)
            .await
            .unwrap();
    }
}
