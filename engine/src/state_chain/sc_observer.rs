use std::sync::Arc;

use anyhow::Result;
use substrate_subxt::{
    events::Raw, system::Phase, Client, EventSubscription, FinalizedEventStorageSubscription,
    RpcClient,
};

use crate::{
    mq::{nats_client::NatsMQClientFactory, IMQClient, IMQClientFactory, Subject, SubjectName},
    settings::{self, Settings},
};

use log::{debug, error, info, trace};

use super::{
    helpers::create_subxt_client,
    runtime::StateChainRuntime,
    sc_event::{raw_event_to_subject, sc_event_from_raw_event},
};

/// Kick off the state chain observer process
pub async fn start(settings: Settings) {
    info!("Begin subscribing to state chain events");

    let mq_client_builder = NatsMQClientFactory::new(&settings.message_queue);

    let mq_client = mq_client_builder.create().await.unwrap();

    let subxt_client = create_subxt_client(settings.state_chain)
        .await
        .expect("Could not create subxt client");

    subscribe_to_events(*mq_client, subxt_client)
        .await
        .expect("Could not subscribe to state chain events");
}

async fn subscribe_to_events<M: 'static + IMQClient>(
    mq_client: M,
    subxt_client: Client<StateChainRuntime>,
) -> Result<()> {
    log::info!("Start subscribing to SC events");
    // subscribe to all finalised events, and then redirect them
    let sub = subxt_client
        .subscribe_finalized_events()
        .await
        .expect("Could not subscribe to state chain events");

    let decoder = subxt_client.events_decoder();

    while let Some(storage_change_set) = sub.next().await {
        for (_key, data) in storage_change_set.changes {
            if let Some(data) = data {
                let raw_events = match decoder.decode_events(&mut &data.0[..]) {
                    Ok(events) => events,
                    Err(error) => return Err(anyhow::Error::msg(error.to_string())),
                };
                for (phase, raw) in raw_events {
                    let raw_event = match raw {
                        Raw::Event(event) => event,
                        Raw::Error(err) => return Err(anyhow::Error::msg(err.to_string())),
                    };

                    log::debug!("SCO received a raw event of: {:?}", raw_event);

                    let subject: Option<Subject> = raw_event_to_subject(&raw_event);

                    if let Some(subject) = subject {
                        let message = sc_event_from_raw_event(raw_event)?;
                        log::debug!("The parsed event is: {:?}", message);
                        match message {
                            Some(event) => {
                                // Publish the message to the message queue
                                match mq_client.publish(subject, &event).await {
                                    Err(err) => {
                                        error!(
                                "Could not publish message `{:?}` to subject `{}`. Error: {}",
                                event,
                                subject.to_subject_name(),
                                err
                            );
                                    }
                                    Ok(_) => trace!("Event: {:#?} pushed to message queue", event),
                                };
                            }
                            None => {
                                debug!(
                                    "Event decoding for an event under subject: {} doesn't exist",
                                    subject.to_subject_name()
                                )
                            }
                        }
                    } else {
                        trace!("Not routing event {:?} to message queue", raw_event);
                    };
                }
            }
        }
    }

    let err_msg = "State Chain Observer stopped subscribing to events!";
    log::error!("{}", err_msg);
    Err(anyhow::Error::msg(err_msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::settings;

    use test_env_log::test;

    #[tokio::test]
    async fn subscribe_finalised_all() {
        let settings = settings::test_utils::new_test_settings().unwrap();
    }

    #[test(tokio::test)]
    async fn start_observer() {
        let settings = settings::test_utils::new_test_settings().unwrap();

        start(settings).await;
    }
}
