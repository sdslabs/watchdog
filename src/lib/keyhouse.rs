extern crate base64;
extern crate crypto;
extern crate serde_json;

use std::time::Duration;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use log::info;
use serde::Deserialize;

use crate::config::Config;
use crate::errors::*;
use crate::logger::LogTarget;

#[derive(Debug, Deserialize)]
struct NameFile {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
}

pub fn validate_user(config: &Config, user: String, ssh_key: &str) -> Result<bool> {
    let name = get_name(config, ssh_key)?;
    info!(target: LogTarget::AUTH.as_str(), "User name: {} ,user {}", name, user);
    if name.trim() != user.trim() {
        info!(target: LogTarget::AUTH.as_str(), "User didn't match with name");
        return Ok(false);
    }
    info!(target: LogTarget::AUTH.as_str(), "User match with name");

    let mut hasher = Sha256::new();
    hasher.input_str(ssh_key);
    let hex = hasher.result_str();
    let host = &config.hostname;

    info!(target: LogTarget::AUTH.as_str(), "Found user hash {}", hex);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    info!(target: LogTarget::AUTH.as_str(), "user {},host {}", user, host);

    let host_url = format!("{}/access/{}?ref=build", config.keyhouse.base_url, host);
    info!(target: LogTarget::AUTH.as_str(), "Host URL: {}", host_url);
    let name_files: Vec<NameFile> = match client
        .get(&host_url)
        .header("Authorization", format!("Bearer {}", config.keyhouse.token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
    {
        Ok(mut r) if r.status().is_success() => {
            let text = r.text()?;
            info!(target: LogTarget::AUTH.as_str(), "Response: {:?}", text);
            serde_json::from_str(&text).unwrap_or_default()
        }
        Ok(r) => {
            info!(target: LogTarget::AUTH.as_str(), "Failed to fetch names: {}", r.status());
            vec![]
        }
        Err(e) => {
            info!(target: LogTarget::AUTH.as_str(), "Error fetching names: {:?}", e);
            vec![]
        }
    };
    let projects: Vec<String> = name_files.into_iter().map(|f| f.name).collect();

    info!(target: LogTarget::AUTH.as_str(), "Found projects: {:?} for host {}", projects, host);
    for project in &projects {
        let project_url = format!(
            "{}/access/{}/{}/{}?ref=build",
            config.keyhouse.base_url, host, project, hex
        );

        match client
            .get(&project_url)
            .header("Authorization", format!("Bearer {}", config.keyhouse.token))
            .send()
        {
            Ok(mut resp) if resp.status().is_success() => {
                let text = resp.text()?;
                info!(target: LogTarget::AUTH.as_str(), "Response: {:?}", text);
                info!(target: LogTarget::AUTH.as_str(), "User validated");
                return Ok(true);
            }
            Ok(_) | Err(_) => continue,
        }
    }

    Ok(false)
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
    String::from_utf8(content).chain_err(|| {
        "Bad UTF8 Encoding. Make sure the file you are trying to access is human readable."
    })
}

pub fn get_name(config: &Config, ssh_key: &str) -> Result<String> {
    let mut hasher = Sha256::new();

    hasher.input_str(ssh_key);
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
                get_content_from_github_json(&json_text)
            } else {
                Ok(String::from("UNKNOWN"))
            }
        }
        Err(e) => Err(Error::from(format!("Unknown reqwest error \n-> {e}"))),
    }
}
