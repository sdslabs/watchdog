extern crate base64;
extern crate crypto;
extern crate reqwest;
extern crate serde_json;

use std::time::Duration;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use reqwest::Client;
use serde_json::Value;

use crate::config::Config;
use crate::{errors::*, logger};

pub fn validate_user(config: &Config, user: String, ssh_key: &str) -> Result<bool> {
    let name = get_name(&config, ssh_key)?;
    logger::logln(&format!("User name: {} ,user {}", name,user));
    if name.trim() != user.trim() {
        logger::logln("User didnt match with name");
        return Ok(false);
    }
    logger::logln("User match with name");
    
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

pub fn fetch_github_projects(config: &Config, user: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let res = client
        .get(&format!(
            "{}/data/hosts/{}?ref=master",
            config.keyhouse.base_url, user
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
                let json: serde_json::Value = serde_json::from_str(&json_text)
                    .map_err(|e| {
                        logger::logln(&format!("Failed to parse JSON from GitHub: {}", e));
                        e
                    })
                    .chain_err(|| "Invalid JSON received from GitHub.")?;

                let encoded_content = json["content"]
                    .as_str()
                    .ok_or_else(|| Error::from("Missing 'content' field in JSON."))?;
                let content = base64::decode(encoded_content.trim_end())
                    .chain_err(|| "Base64 decoding failed.")?;
                let decoded_str =
                    String::from_utf8(content).chain_err(|| "UTF-8 decoding failed.")?;
                logger::logln(&format!("Decoded string: {}", decoded_str));
                let projects = decoded_str
                    .lines()
                    .filter_map(|line| line.split('|').nth(1))
                    .map(String::from)
                    .collect::<Vec<String>>();
                logger::logln(&format!("Projects: {:?}", projects));
                Ok(projects)
            } else {
                logger::logln(&format!(
                    "GitHub API request failed with status: {}",
                    r.status()
                ));
                Err(Error::from(format!(
                    "GitHub API request failed with status: {}",
                    r.status()
                )))
            }
        }
        Err(e) => Err(Error::from(format!("Unknown reqwest error \n-> {}", e))),
    }
}

pub fn fetch_file_names(
    base_url: &str,
    directory: &str,
    token: &str,
    file_names: &mut Vec<String>,
) -> Result<()> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    println!(
        "Fetching file names from {}/{}?ref=master and token {}",
        base_url, directory, token
    );
    let mut response = client
        .get(&format!("{}/{}?ref=master", base_url, directory))
        .header("Authorization", &format!("Bearer {}", token))
        .send()?;
    
    if response.status().is_success() {
        let contents: Value = response.json()?;
        if let Some(files) = contents.as_array() {
            for file in files {
                if let Some(file_name) = file["name"].as_str() {
                    file_names.push(file_name.to_string());
                }
            }
        }
    } else {
        return Err(format!(
            "GitHub API request failed with status: {}",
            response.status()
        )
        .into());
    }

    println!("Fetched file names: {:?}", file_names);
    Ok(())
}