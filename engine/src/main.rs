use log::info;

use chainflip_engine::{eth, mq::Options, sc_observer, settings::Settings, witness};

#[tokio::main]
async fn main() {
    // init the logger
    env_logger::init();

    let settings = Settings::new().expect("Failed to initialise settings");

    // set up the message queue
    let mq_options = Options {
        url: format!(
            "{}:{}",
            settings.message_queue.hostname, settings.message_queue.port
        ),
    };

    info!("Start the engines! :broom: :broom: ");

    sc_observer::sc_observer::start(mq_options.clone(), settings.clone().state_chain).await;

    eth::start(settings).await;

    // start witnessing other chains
    witness::witness::start(mq_options).await;
}
