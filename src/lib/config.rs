use std::fs;

use serde_derive::Deserialize;
use toml_edit::{value, Document};

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

pub fn set_config_value(key: &str, val: &str) -> Result<()> {
    let toml_str = fs::read_to_string("/opt/watchdog/config.toml")?;
    let mut doc = toml_str.parse::<Document>().chain_err(|| {
        "Invalid TOML file. Please reverify if /opt/watchdog/config.toml is a valid toml file."
    })?;
    match key {
        "hostname" => {
            doc["hostname"] = value(val);
        }
        "keyhouse.base_url" => {
            doc["keyhouse"]["base_url"] = value(val);
        }
        "keyhouse.token" => {
            doc["keyhouse"]["token"] = value(val);
        }
        "notifiers.slack" => {
            doc["notifiers"]["slack"] = value(val);
        }
        _ => {
            return Err("Invalid Key passed".into());
        }
    }
    fs::write("/opt/watchdog/config.toml", doc.to_string())?;
    Ok(())
}

pub fn get_config_value(key: &str) -> Result<String> {
    let toml_str = fs::read_to_string("/opt/watchdog/config.toml")?;
    let doc = toml_str.parse::<Document>().chain_err(|| {
        "Invalid TOML file. Please reverify if /opt/watchdog/config.toml is a valid toml file."
    })?;
    let val = match key {
        "hostname" => doc["hostname"].as_str(),
        "keyhouse.base_url" => doc["keyhouse"]["base_url"].as_str(),
        "keyhouse.token" => doc["keyhouse"]["token"].as_str(),
        "notifiers.slack" => doc["notifiers"]["slack"].as_str(),
        _ => {
            return Err("Invalid Key passed".into());
        }
    };
    return match val {
        Some(s) => Ok(String::from(s)),
        None => Err("config.toml file doesn't contain that key.".into()),
    };
}
