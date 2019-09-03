use serde_derive::Deserialize;
use std::fs;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub slack_api_url: String,
    pub keyhouse_hostname: String,
    pub keyhouse_token: String,
    pub keyhouse_base_url: String,
    pub temp_env_file: String,
    pub watchdog_base_url: String,
}

pub fn read_config() -> Config {
	let toml_str = fs::read_to_string("/home/kanav/projects/watchdog-rs/config.toml")
						.expect("Error reading the config.toml file.");
	let config: Config = toml::from_str(&toml_str).unwrap();
	return config;  
}
