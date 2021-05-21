use chainflip_common::types::addresses::EthereumAddress;
use config::{Config, ConfigError, File};
use std::env;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct MessageQueue {
    pub hostname: String,
    pub port: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StateChain {
    pub hostname: String,
    pub port: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Eth {
    pub hostname: String,
    pub port: u32,

    // TODO: Into an Ethereum Address type?
    pub stake_manager_eth_address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub message_queue: MessageQueue,
    pub state_chain: StateChain,
    pub eth: Eth,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default.toml"))?;

        // Add in the current environment file
        // Default to testing env - this is optional, without it, it'll just use the default
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "testing".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }

    /// Creating the settings to be used for tests
    // TODO: cfg test ?
    pub fn new_test() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/testing.toml"))?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn init_config() {
        let settings = Settings::new();
        assert!(settings.is_ok());

        let settings = settings.unwrap();
        assert_eq!(settings.message_queue.hostname, "localhost");
    }

    #[test]
    #[ignore = "testing isn't any different to default, so no point testing it at the moment"]
    fn test_init_config_with_testing_config() {
        todo!();
    }
}
