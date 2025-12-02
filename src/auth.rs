use lib::logger::LogTarget;
use log::{error, info};
use nix::unistd::{fork, ForkResult};

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use lib::config::read_config;
use lib::config::Config;
use lib::errors::*;
use lib::init::init;
use lib::keyhouse::{get_name, validate_user};
use lib::notifier;
use log::warn;
use std::fs;
use std::path::Path;

#[cfg(feature = "auto-update")]
use crate::update::handle_update;

pub fn handle_auth(ssh_host_username: &str, ssh_key: &str) -> Result<()> {
    let config = read_config()?;
    init(&config)?;
    info!(target: LogTarget::AUTH.as_str(), "ssh_key in handle_auth: {}", ssh_key);

    #[cfg(feature = "auto-update")]
    {
        match handle_update() {
            Ok(_) => {
                info!(target: LogTarget::UPDATE.as_str(), "Update handled successfully");
            }
            Err(e) => {
                error!(target: LogTarget::UPDATE.as_str(), "Error handling update: {}", e);
                return Err(e);
            }
        }
    }

    match validate_user(&config, ssh_host_username.to_string(), ssh_key) {
        Ok(true) => {
            info!(target: LogTarget::AUTH.as_str(), "User validated by handle auth");
            println!("{}", ssh_key);
            Ok(())
        }

        Ok(false) => {
            info!(target: LogTarget::AUTH.as_str(), "User not validated");
            let name = get_name(&config, ssh_key)?;
            info!(target: LogTarget::AUTH.as_str(), "Logging failed");
            match fork() {
                Ok(ForkResult::Parent { .. }) => {}
                Ok(ForkResult::Child) => {
                    notifier::post_ssh_summary(
                        &config,
                        false,
                        &name,
                        &ssh_host_username.to_string(),
                    )?;
                    std::process::exit(0);
                }
                Err(_) => println!("Fork failed"),
            }
            Ok(())
        }
        Err(e) => {
            warn!(target: LogTarget::AUTH.as_str(), "Error validating from Keyhouse (GitHub): {}. Checking local cache...", e);

            if check_local_cache(&config, ssh_host_username, ssh_key) {
                info!(target: LogTarget::AUTH.as_str(), "User validated from local cache");
                println!("{}", ssh_key);
                Ok(())
            } else {
                error!(target: LogTarget::AUTH.as_str(), "User not found in local cache or access denied.");
                Err(e).chain_err(|| "Error while validating user from keyhouse and cache miss")
            }
        }
    }
}

// Helper function to validate user against the local cache
fn check_local_cache(config: &Config, user: &str, ssh_key: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.input_str(ssh_key);
    let hex = hasher.result_str();

    let cache_path = &config.cache_path;

    let name_path = format!("{}/names/{}", cache_path, hex);
    if !Path::new(&name_path).exists() {
        warn!(target: LogTarget::AUTH.as_str(), "Cache Miss: Name file not found for hash {}", hex);
        return false;
    }

    let cached_user = match fs::read_to_string(&name_path) {
        Ok(u) => u.trim().to_string(),
        Err(_) => return false,
    };

    if cached_user != user {
        warn!(target: LogTarget::AUTH.as_str(), "Cache Mismatch: User '{}' requested, but key hash belongs to '{}'", user, cached_user);
        return false;
    }

    let host_access_path = format!("{}/access/{}", cache_path, config.hostname);
    if !Path::new(&host_access_path).exists() {
        warn!(target: LogTarget::AUTH.as_str(), "Cache Miss: No access directory for host '{}'", config.hostname);
        return false;
    }

    match fs::read_dir(host_access_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let hash_path = entry.path().join(&hex);
                    if hash_path.exists() {
                        info!(target: LogTarget::AUTH.as_str(), "Cache Hit: User '{}' has access via group {:?}", user, entry.file_name());
                        return true;
                    }
                }
            }
        }
        Err(e) => {
            warn!(target: LogTarget::AUTH.as_str(), "Cache Error: Failed to read access directory: {}", e);
            return false;
        }
    }

    warn!(target: LogTarget::AUTH.as_str(), "Cache Miss: User '{}' found in names, but no access file found for host '{}'", user, config.hostname);
    false
}