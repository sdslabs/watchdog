use serde_derive::Deserialize;
use std::fs;

#[derive(Deserialize, Clone)]
pub struct TempEnvirontment {
    pub ssh_host_username: String,
    pub ssh_key: String,
}

pub fn read_temp_env(path: String) -> TempEnvirontment {
    let toml_str = fs::read_to_string(&path).expect("Error reading the environment toml file.");
    let env: TempEnvirontment = toml::from_str(&toml_str).unwrap();
    return env;
}
