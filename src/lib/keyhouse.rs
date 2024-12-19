extern crate base64;
extern crate crypto;
extern crate reqwest;
extern crate serde_json;

use std::time::Duration;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use crate::config::Config;
use crate::errors::*;

pub fn validate_user(config: &Config, user: String, ssh_key: &str) -> Result<bool> {
    let mut hasher = Sha256::new();

    hasher.input_str(&ssh_key);
    let hex = hasher.result_str();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let res = client
        .get(&format!(
            "{}/access/{}/{}/{}?ref=build",
            config.keyhouse.base_url, config.hostname, user, hex
        ))
        .header(
            "Authorization",
            &format!("Bearer {}", config.keyhouse.token),
        )
        .send();

    match res {
        Ok(r) => {
            if r.status().is_success() {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        Err(e) => Err(Error::from(format!("Unknown reqwest error \n-> {}", e))),
    }
}

fn get_content_from_github_json(json_text: &str) -> Result<String> {
    let json: serde_json::Value = serde_json::from_str(json_text)
                                    .chain_err(|| "Invalid JSON recieved from GitHub. Probably GitHub is facing some issues. Check https://githubstatus.com.")?;
    let encoded_content = json["content"]
        .as_str()
        .ok_or(Error::from(""))
        .chain_err(|| "No key 'content' found in JSON recieved from GitHub.")?;
    let _len = str::len(encoded_content);
    let content = base64::decode(&encoded_content.trim_end())
                    .chain_err(|| "Bad Base64 Encoding. Probably GitHub is facing some issues. Check https://githubstatus.com.")?;
    Ok(String::from_utf8(content).chain_err(|| {
        "Bad UTF8 Encoding. Make sure the file you are trying to access is human readable."
    })?)
}

pub fn get_name(config: &Config, ssh_key: &str) -> Result<String> {
    let mut hasher = Sha256::new();

    hasher.input_str(&ssh_key);
    let hex = hasher.result_str();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let res = client
        .get(&format!(
            "{}/names/{}?ref=build",
            config.keyhouse.base_url, hex
        ))
        .header(
            "Authorization",
            &format!("Bearer {}", config.keyhouse.token),
        )
        .send();

    match res {
        Ok(mut r) => {
            if r.status().is_success() {
                let json_text = r.text()?;
                return get_content_from_github_json(&json_text);
            } else {
                return Ok(String::from("UNKNOWN"));
            }
        }
        Err(e) => Err(Error::from(format!("Unknown reqwest error \n-> {}", e))),
    }
}
