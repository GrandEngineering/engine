use std::{fs, io::Error};

use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigTomlServer {
    pub cgrpc_token: Option<String>, // Administrator Token, used to invoke cgrpc reqs. If not preset will default to no protection.
    pub port: String,
    pub clean_tasks: u64,
}
impl Default for ConfigTomlServer {
    fn default() -> Self {
        Self {
            port: "[::1]:50051".into(),
            cgrpc_token: None,
            clean_tasks: 60,
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub config_toml: ConfigTomlServer,
}

impl Config {
    #[allow(clippy::new_without_default)]
    #[instrument]
    pub fn new() -> Self {
        let mut content: String = "".to_owned();
        let result: Result<String, Error> = fs::read_to_string("config.toml");
        if result.is_ok() {
            content = result.unwrap();
        };
        let config_toml: ConfigTomlServer = toml::from_str(&content).unwrap_or_else(|err| {
            error!("Failed to parse config file.");
            error!("{:#?}", err);
            ConfigTomlServer::default()
        });
        Self { config_toml }
    }
}
