use std::fs;

use serde_derive::Deserialize;

use crate::errors::*;

#[derive(Deserialize, Clone)]
pub struct KeyhouseConf {
    pub base_url: String,
    pub token: String,
}

#[derive(Deserialize, Clone)]
pub struct NotifiersConf {
    pub slack: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub hostname: String,
    pub keyhouse: KeyhouseConf,
    pub notifiers: NotifiersConf,
}

pub fn read_config() -> Result<Config> {
    let toml_str = fs::read_to_string("/opt/watchdog/config.toml")?;
    let config: Config = toml::from_str(&toml_str)?;
    Ok(config)
}
