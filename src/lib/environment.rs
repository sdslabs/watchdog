use std::fs;

use serde_derive::Deserialize;

use crate::errors::*;

#[derive(Deserialize, Clone)]
pub struct TempEnvironment {
    pub ssh_host_username: String,
    pub ssh_key: String,
}

pub fn read_temp_env(path: &str) -> Result<TempEnvironment> {
    let toml_str = fs::read_to_string(path)?;
    let env: TempEnvironment = toml::from_str(&toml_str)?;
    Ok(env)
}
