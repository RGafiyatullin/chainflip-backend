use std::time::Duration;

use anyhow::Context;
use chainflip_engine::{
    eth::{
        self, build_broadcast_channel,
        rpc::{EthDualRpcClient, EthHttpRpcClient, EthWsRpcClient},
    },
    logging,
    settings::{CommandLineOptions, Settings},
    task_scope::with_main_task_scope,
};
use futures::FutureExt;
use state_chain_runtime::CfeSettings;

#[test]
pub fn test_witnessing_1() -> anyhow::Result<()> {
    let settings =
        Settings::from_file_and_env("config/Testing.toml", CommandLineOptions::default()).unwrap();

    let root_logger = logging::utils::new_json_logger_with_tag_filter(
        settings.log.whitelist.clone(),
        settings.log.blacklist.clone(),
    );

    slog::info!(root_logger, "Start the engines! :broom: :broom: ");

    with_main_task_scope(|scope| {
        async {
            // Init web3 and eth broadcaster before connecting to SC, so we can diagnose these config errors, before
            // we connect to the SC (which requires the user to be staked)
            let eth_ws_rpc_client = EthWsRpcClient::new(&settings.eth, &root_logger)
                .await
                .context("Failed to create EthWsRpcClient")?;

            let eth_http_rpc_client = EthHttpRpcClient::new(&settings.eth, &root_logger)
                .context("Failed to create EthHttpRpcClient")?;

            let eth_dual_rpc =
                EthDualRpcClient::new(eth_ws_rpc_client, eth_http_rpc_client, &root_logger);

            let (witnessing_instruction_sender, [witnessing_instruction_receiver]) =
                build_broadcast_channel(10);

            let cfe_settings = CfeSettings::default();

            let (_cfe_settings_update_sender, cfe_settings_update_receiver) =
                tokio::sync::watch::channel(cfe_settings);

            scope.spawn(eth::start_chain_data_witnesser(
                eth_dual_rpc,
                witnessing_instruction_receiver,
                cfe_settings_update_receiver,
                Duration::from_secs(1),
                &root_logger,
            ));

            witnessing_instruction_sender.send(eth::ObserveInstruction::Start(0, 0))?;

            Ok(())
        }
        .boxed()
    })
}
