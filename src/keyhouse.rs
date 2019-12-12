extern crate base64;
extern crate crypto;
extern crate reqwest;

use crate::config;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use log::error;
use serde_json;

pub fn validate_user(config: &config::Config, user: String, ssh_key: &String) -> bool {
    let mut hasher = Sha256::new();

    hasher.input_str(&ssh_key);
    let hex = hasher.result_str();

    let res = reqwest::get(&format!(
        "{}/access/{}/{}/{}?ref=build&access_token={}",
        config.keyhouse_base_url, config.keyhouse_hostname, user, hex, config.keyhouse_token
    ));

    match res {
        Ok(r) => {
            if r.status().is_success() {
                return true;
            } else {
                return false;
            }
        }
        Err(_) => {
            error!("Error making request to keyhouse.");
            return false;
        }
    }
}

pub fn get_name(config: &config::Config, ssh_key: String) -> String {
    let mut hasher = Sha256::new();

    hasher.input_str(&ssh_key);
    let hex = hasher.result_str();

    let res = reqwest::get(&format!(
        "{}/names/{}?ref=build&access_token={}",
        config.keyhouse_base_url, hex, config.keyhouse_token
    ));

    match res {
        Ok(mut r) => {
            if r.status().is_success() {
                let json_text = r.text().unwrap();
                let json: serde_json::Value = serde_json::from_str(&json_text).unwrap();
                let name = json["content"].as_str().unwrap();
                let len = String::len(&String::from(name));
                return String::from_utf8(base64::decode(&name[..len - 2]).unwrap()).unwrap();
            } else {
                return String::new();
            }
        }
        Err(_) => {
            error!("Error while making a request to keyhouse");
            return String::new();
        }
    }
}
