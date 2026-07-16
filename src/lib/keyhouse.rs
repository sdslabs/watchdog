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

fn canonicalize_ssh_key(ssh_key: &str) -> Option<String> {
    let mut parts = ssh_key.split_whitespace();
    let key_type = parts.next()?;
    let key = parts.next()?;

    Some(format!("{key_type} {key}"))
}

fn parse_user_keys(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(canonicalize_ssh_key)
        .collect()
}

fn hash_ssh_key(ssh_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(ssh_key);
    hasher.result_str()
}

fn hashes_for_matching_key(user_keys: &[String], ssh_key: &str) -> Option<Vec<String>> {
    let requested_key = canonicalize_ssh_key(ssh_key)?;
    if !user_keys.iter().any(|key| key == &requested_key) {
        return None;
    }

    Some(user_keys.iter().map(|key| hash_ssh_key(key)).collect())
}

pub fn validate_user(config: &Config, user: String, ssh_key: &str) -> Result<bool> {
    let name = get_name(config, ssh_key)?;
    info!(target: LogTarget::AUTH.as_str(), "User name: {} ,user {}", name, user);

    // Keep the generated build-branch lookup as the fast path. Additional keys
    // are resolved from the user's source key file when no generated name exists.
    let hashes = if name.trim() == user.trim() {
        info!(target: LogTarget::AUTH.as_str(), "User match with name");
        vec![hash_ssh_key(ssh_key)]
    } else {
        info!(target: LogTarget::AUTH.as_str(), "Generated name didn't match user; checking the user's key list");

        let user_keys = get_user_keys(config, user.trim())?;
        match hashes_for_matching_key(&user_keys, ssh_key) {
            Some(hashes) => {
                info!(target: LogTarget::AUTH.as_str(), "Key found in user key list");
                hashes
            }
            None => {
                info!(target: LogTarget::AUTH.as_str(), "Key not found in user key list");
                return Ok(false);
            }
        }
    };
    let host = &config.hostname;

    info!(target: LogTarget::AUTH.as_str(), "Found user hashes {:?}", hashes);

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
        // Access is granted to the user, so any registered key hash for that user
        // may carry the generated access marker.
        for hash in &hashes {
            let project_url = format!(
                "{}/access/{}/{}/{}?ref=build",
                config.keyhouse.base_url, host, project, hash
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
    let encoded_content: String = encoded_content
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect();
    let content = base64::decode(&encoded_content)
                    .chain_err(|| "Bad Base64 Encoding. Probably GitHub is facing some issues. Check https://githubstatus.com.")?;
    String::from_utf8(content).chain_err(|| {
        "Bad UTF8 Encoding. Make sure the file you are trying to access is human readable."
    })
}

fn get_user_keys(config: &Config, user: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let response = client
        .get(&format!("{}/data/keys/{}", config.keyhouse.base_url, user))
        .header(
            "Authorization",
            &format!("Bearer {}", config.keyhouse.token),
        )
        .header("Accept", "application/vnd.github.v3+json")
        .send();

    match response {
        Ok(mut response) if response.status().is_success() => {
            let json_text = response.text()?;
            let content = get_content_from_github_json(&json_text)?;
            Ok(parse_user_keys(&content))
        }
        Ok(response) => {
            info!(target: LogTarget::AUTH.as_str(), "Failed to fetch key list for user {}: {}", user, response.status());
            Ok(vec![])
        }
        Err(error) => Err(Error::from(format!(
            "Unknown reqwest error while fetching keys for user {user}\n-> {error}"
        ))),
    }
}

pub fn get_name(config: &Config, ssh_key: &str) -> Result<String> {
    let hex = hash_ssh_key(ssh_key);

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

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_KEY: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICsfZlzGJSe6B5q5mEp9E2blI/xW7mW34h1xdHxFUQHs";
    const SECOND_KEY: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICUKyjr6icfamUc4l7tPFr82VgSf373rciLGcEGPYirv";

    #[test]
    fn parses_multiple_keys_and_ignores_comments_and_blank_lines() {
        let content = format!(
            "# laptop and workstation\n{FIRST_KEY} user1_mail1@sdslabs.co\n\n  {SECOND_KEY}\tuser1_mail2@sdslabs.co\r\n"
        );

        assert_eq!(
            parse_user_keys(&content),
            vec![FIRST_KEY.to_string(), SECOND_KEY.to_string()]
        );
    }

    #[test]
    fn skips_malformed_key_lines() {
        let content = format!("missing-key-body\n# {FIRST_KEY}\n{SECOND_KEY}\n");

        assert_eq!(parse_user_keys(&content), vec![SECOND_KEY.to_string()]);
    }

    #[test]
    fn decodes_github_content_with_wrapped_base64() {
        let content =
            format!("{FIRST_KEY} user1_mail1@sdslabs.co\n{SECOND_KEY} user1_mail2@sdslabs.co\n");
        let encoded = base64::encode(&content);
        let wrapped = encoded
            .as_bytes()
            .chunks(60)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        let json = serde_json::json!({ "content": wrapped }).to_string();

        assert_eq!(get_content_from_github_json(&json).unwrap(), content);
    }

    #[test]
    fn selects_all_user_hashes_for_a_secondary_key() {
        let first_hash = hash_ssh_key(FIRST_KEY);
        let second_hash = hash_ssh_key(SECOND_KEY);
        let user_keys = vec![FIRST_KEY.to_string(), SECOND_KEY.to_string()];

        let hashes = hashes_for_matching_key(&user_keys, SECOND_KEY).unwrap();

        assert_eq!(hashes, vec![first_hash, second_hash]);
    }

    #[test]
    fn rejects_a_key_that_is_not_in_the_users_key_list() {
        let user_keys = vec![FIRST_KEY.to_string()];

        assert!(hashes_for_matching_key(&user_keys, SECOND_KEY).is_none());
    }
}
