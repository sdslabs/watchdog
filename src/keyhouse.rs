use crate::config;
extern crate base64;
extern crate crypto;
extern crate reqwest;
use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;
use serde_json;

pub fn validate_user(config: &config::Config, user: String, ssh_key: &String) -> bool {
    let mut hasher = Sha256::new();
    hasher.input_str(&ssh_key);
    let hex = hasher.result_str();
    let client = reqwest::Client::new();
    let res = client
        .get(&format!(
            "{}/access/{}/{}/{}?ref=build&access_token={}",
            config.keyhouse_base_url, config.keyhouse_hostname, user, hex, config.keyhouse_token
        ))
        .send();
    if res.unwrap().status().is_success() {
        return true;
    } else {
        return false;
    }
}

pub fn get_name(config: &config::Config, ssh_key: String) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(&ssh_key);
    let hex = hasher.result_str();
    let mut res = reqwest::get(&format!(
        "{}/names/{}?ref=build&access_token={}",
        config.keyhouse_base_url, hex, config.keyhouse_token
    ))
    .unwrap();
    if res.status().is_success() {
        let json = res.text().unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let name = v["content"].as_str().unwrap();
        let len = String::len(&String::from(name));
        return String::from_utf8(base64::decode(&name[..len - 2]).unwrap()).unwrap();
    } else {
        return String::new();
    }
}
