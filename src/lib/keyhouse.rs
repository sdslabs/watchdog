extern crate base64;
extern crate crypto;
extern crate serde_json;

use std::collections::HashMap;
use std::time::Duration;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use log::{debug, info};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::config::Config;
use crate::errors::*;

#[derive(Debug, Deserialize)]
struct NameFile {
    name: String,
}

pub fn validate_user(config: &Config, user: String, ssh_key: &str) -> Result<bool> {
    let name = get_name(&config, ssh_key)?;
    info!(target: "auth", "User name: {} ,user {}", name, user);
    if name.trim() != user.trim() {
        info!(target: "auth", "User didn't match with name");
        return Ok(false);
    }
    info!(target: "auth", "User match with name");

    let mut hasher = Sha256::new();
    hasher.input_str(&ssh_key);
    let hex = hasher.result_str();
    let host = &config.hostname;

    info!(target: "auth", "Found user hash {}", hex);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    info!(target: "auth", "user {},host {}", user, host);

    let host_url = format!("{}/access/{}?ref=build", config.keyhouse.base_url, host);
    info!(target: "auth", "Host URL: {}", host_url);
    let name_files: Vec<NameFile> = match client
        .get(&host_url)
        .header("Authorization", format!("Bearer {}", config.keyhouse.token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
    {
        Ok(mut r) if r.status().is_success() => {
            let text = r.text()?;
            info!(target: "auth", "Response: {:?}", text);
            serde_json::from_str(&text).unwrap_or_default()
        }
        Ok(r) => {
            info!(target: "auth", "Failed to fetch names: {}", r.status());
            vec![]
        }
        Err(e) => {
            info!(target: "auth", "Error fetching names: {:?}", e);
            vec![]
        }
    };
    let projects: Vec<String> = name_files.into_iter().map(|f| f.name).collect();

    info!(target: "auth", "Found projects: {:?} for host {}", projects, host);
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
                info!(target: "auth", "Response: {:?}", text);
                info!(target: "auth", "User validated");
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

pub fn fetch_github_projects(config: &Config, key_hash: &str) -> Result<Vec<String>> {
    debug!(target: "update", "Fetching projects for key hash: {}", key_hash);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let host_url = format!(
        "{}/access/{}?ref=build",
        config.keyhouse.base_url, config.hostname
    );

    let name_files: Vec<NameFile> = match client
        .get(&host_url)
        .header("Authorization", format!("Bearer {}", config.keyhouse.token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
    {
        Ok(mut r) if r.status().is_success() => {
            let text = r.text()?;
            info!(target: "update", "Response: {:?}", text);
            serde_json::from_str(&text).unwrap_or_default()
        }
        Ok(r) => {
            info!(target: "update", "Failed to fetch names: {}", r.status());
            vec![]
        }
        Err(e) => {
            info!(target: "update", "Error fetching names: {:?}", e);
            vec![]
        }
    };
    debug!(target: "update", "Fetched names: {:?}", name_files);
    let projects: Vec<String> = name_files.into_iter().map(|f| f.name).collect();
    debug!(target: "update", "Found projects: {:?} for host {}", projects, config.hostname);
    let mut user_projects: Vec<String> = Vec::new();
    for project in &projects {
        let project_url = format!(
            "{}/access/{}/{}/{}?ref=build",
            config.keyhouse.base_url, config.hostname, project, key_hash
        );

        match client
            .get(&project_url)
            .header("Authorization", format!("Bearer {}", config.keyhouse.token))
            .send()
        {
            Ok(mut resp) if resp.status().is_success() => {
                let text = resp.text()?;
                info!(target: "auth", "Response: {:?}", text);
                user_projects.push(project.to_string());
            }
            Ok(_) | Err(_) => continue,
        }
    }
    Ok(user_projects)
}

pub fn fetch_file_names(
    base_url: &str,
    directory: &str,
    token: &str,
    user_to_key: &mut HashMap<String, String>,
) -> Result<()> {
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    let list_url = format!("{}/{}?ref=build", base_url, directory);
    let mut response = client
        .get(&list_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    let files: Value = response.json()?;

    if let Some(entries) = files.as_array() {
        for entry in entries {
            if let Some(key_hash) = entry["name"].as_str() {
                let file_url = format!("{}/{}/{}?ref=build", base_url, directory, key_hash);

                let mut file_response = client
                    .get(&file_url)
                    .header("Authorization", format!("Bearer {}", token))
                    .send()?;

                if file_response.status().is_success() {
                    let file_json: Value = file_response.json()?;

                    if let (Some(encoded), Some(encoding)) = (
                        file_json.get("content").and_then(|v| v.as_str()),
                        file_json.get("encoding").and_then(|v| v.as_str()),
                    ) {
                        if encoding == "base64" {
                            let cleaned = encoded.replace('\n', "").replace('\r', "");
                            let decoded_bytes = base64::decode(&cleaned)?;
                            let username = String::from_utf8(decoded_bytes)?.trim().to_string();

                            user_to_key.insert(username, key_hash.to_string());
                        }
                    }
                }
            }
        }
    }
    info!(target: "update", "Fetched user to key mapping: {:?}", user_to_key);

    Ok(())
}
