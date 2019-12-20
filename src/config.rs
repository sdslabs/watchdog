use serde_derive::Deserialize;
use std::fs;
use crate::errors::*;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub slack_api_url: String,
    pub keyhouse_hostname: String,
    pub keyhouse_token: String,
    pub keyhouse_base_url: String,
    pub temp_env_file: String,
    pub watchdog_base_url: String,
    pub error_log_file: String,
}

pub fn read_config() -> Result<Config> {
    let toml_str = fs::read_to_string("/opt/watchdog/config.toml")?;
    let config: Config = toml::from_str(&toml_str)?;
    Ok(config)
}
