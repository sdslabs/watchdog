use std::collections::HashMap;
use std::fs;

use lib::config::read_config;
use lib::config::Config;
use lib::constants::BASE_COMMIT_PATH;
use lib::errors::*;
use lib::init::init;
use lib::keyhouse::fetch_and_decode_file;
use lib::keyhouse::fetch_diff;
use lib::keyhouse::fetch_file_names;
use lib::keyhouse::fetch_github_projects;
use lib::keyhouse::fetch_recent_commit;
use lib::logger::LogTarget;
use lib::utils::add_user_to_groups;
use lib::utils::create_linux_user;
use lib::utils::delete_user;
use lib::utils::extract_diff_parts;
use lib::utils::remove_user_from_groups;
use lib::utils::update_user_bashrc;
use lib::utils::user_exists;
use log::debug;
use log::{error, info};

pub fn handle_update() -> Result<()> {
    let mut user_to_key: HashMap<String, String> = HashMap::new();
    log::info!(target: LogTarget::UPDATE.as_str(), "Watchdog update triggered.");
    let config = read_config()?;
    init(&config)?;

    let base_commit = fs::read_to_string(BASE_COMMIT_PATH)
        .unwrap_or_default()
        .trim()
        .to_string();

    if !base_commit.is_empty() {
        info!(target: LogTarget::UPDATE.as_str(), "Found base_commit: {}. Running incremental update.", base_commit);
        match update_from_commit(&config, &base_commit) {
            Ok(_) => {
                info!(target: LogTarget::UPDATE.as_str(), "Incremental update completed successfully.");
            }
            Err(e) => {
                error!(target: LogTarget::UPDATE.as_str(), "Incremental update failed: {}", e);
                return Err(e);
            }
        }
        return Ok(());
    }

    let _ = fetch_file_names(
        &config.keyhouse.base_url,
        "names",
        &config.keyhouse.token,
        &mut user_to_key,
    )?;
    info!(target: LogTarget::UPDATE.as_str(), "Fetched users: {:?}", user_to_key);
    debug!("Fetched users: {:?}", user_to_key);
    for (user, key_hash) in user_to_key.iter() {
        if !user_exists(user) {
            match create_linux_user(user) {
                Ok(_) => {
                    info!(target: LogTarget::UPDATE.as_str(), "User {} created successfully.", user);
                }
                Err(e) => {
                    error!(target: LogTarget::UPDATE.as_str(), "Failed to create user {}: {}", user, e);
                    continue;
                }
            }

            match update_user_bashrc(user) {
                Ok(_) => {
                    info!(target: LogTarget::UPDATE.as_str(), "User {} bashrc updated successfully.", user);
                }
                Err(e) => {
                    error!(target: LogTarget::UPDATE.as_str(), "Failed to update user {} bashrc: {}", user, e);
                }
            }
        } else {
            info!(target: LogTarget::UPDATE.as_str(), "User {} already exists. Skipping creation and bashrc update.", user);
        }
        debug!("User: {}, Key Hash: {}", user, key_hash);
        match fetch_github_projects(&config, key_hash) {
            Ok(projects) => {
                info!(target: LogTarget::UPDATE.as_str(), "Fetched projects for {}: {:?}", user, projects);
                if let Err(e) = add_user_to_groups(user, &projects) {
                    info!(target: LogTarget::UPDATE.as_str(), "Failed to add user {} to project groups: {}", user, e);
                }
            }
            Err(e) => {
                error!(target: LogTarget::UPDATE.as_str(), "Failed to fetch projects for user {}: {}", user, e);
            }
        }
    }
    Ok(())
}

pub fn update_from_commit(config: &Config, base_commit: &str) -> Result<()> {
    info!(target: LogTarget::UPDATE.as_str(), "Performing incremental update from commit: {}", base_commit);
    let merge_commit = match fetch_recent_commit(config) {
        Ok(commit) => {
            info!(target: LogTarget::UPDATE.as_str(), "Fetched merge commit: {}", commit);
            commit
        }
        Err(e) => {
            error!(target: LogTarget::UPDATE.as_str(), "Failed to fetch merge commit: {}", e);
            return Err(e);
        }
    };
    let diff = match fetch_diff(&config, &base_commit, &merge_commit) {
        Ok(diff) => {
            info!(target: LogTarget::UPDATE.as_str(), "Fetched diff: {}", diff);
            diff
        }
        Err(e) => {
            error!(target: LogTarget::UPDATE.as_str(), "Failed to fetch diff: {}", e);
            return Err(e);
        }
    };
    for (cloud_provider, project, hash, status) in extract_diff_parts(&diff) {
        info!(
            "Parsed diff - Project: {}, Cloud Provider: {}, Hash: {}, Status: {}",
            project, cloud_provider, hash, status
        );
        if cloud_provider != config.hostname {
            info!(target: LogTarget::UPDATE.as_str(), "Skipping cloud provider {} as it does not match the config hostname {}", cloud_provider, config.hostname);
            continue;
        }
        if let Ok(Some(decoded_str)) = fetch_and_decode_file(&config, &hash, &status, &base_commit)
        {
            info!("Decoded file for hash {}", hash);
            if status == "added" {
                info!("Adding user to group...");
                add_user_to_groups(&decoded_str, &[project.clone()]).unwrap_or_else(|e| {
                    error!("Failed to add user to group: {}", e);
                });
            } else if status == "deleted" {
                info!("Removing user from group...");
                remove_user_from_groups(&decoded_str, &[project.clone()]).unwrap_or_else(|e| {
                    error!("Failed to remove user from group: {}", e);
                });
            } else if status == "deleteduser" {
                info!("Deleting user...");
                delete_user(&decoded_str).unwrap_or_else(|e| {
                    error!("Failed to delete user: {}", e);
                });
            }
        }
    }
    info!(target: LogTarget::UPDATE.as_str(), "Incremental update completed.");
    fs::write(BASE_COMMIT_PATH, merge_commit.clone())?;
    info!(target: LogTarget::UPDATE.as_str(), "Updated base commit to: {}", merge_commit);
    Ok(())
}
