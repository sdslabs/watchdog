use std::collections::HashMap;

use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::keyhouse::fetch_file_names;
use lib::keyhouse::fetch_github_projects;
use lib::utils::add_user_to_groups;
use lib::utils::create_linux_user;
use log::debug;
use log::{error, info};

pub fn handle_update() -> Result<()> {
    let mut user_to_key: HashMap<String, String> = HashMap::new();
    log::info!(target: "update", "Watchdog update triggered.");
    let config = read_config()?;
    init(&config)?;
    let _ = fetch_file_names(
        &config.keyhouse.base_url,
        "names",
        &config.keyhouse.token,
        &mut user_to_key,
    )?;
    info!(target: "update", "Fetched users: {:?}", user_to_key);
    debug!("Fetched users: {:?}", user_to_key);
    for (user, key_hash) in user_to_key.iter() {
        match create_linux_user(user) {
            Ok(_) => {
                info!(target: "update", "User {} created successfully.", user);
            }
            Err(e) => {
                error!(target: "update", "Failed to create user {}: {}", user, e);
            }
        }
        debug!("User: {}, Key Hash: {}", user, key_hash);
        match fetch_github_projects(&config, key_hash) {
            Ok(projects) => {
                info!(target: "update", "Fetched projects for {}: {:?}", user, projects);
                if let Err(e) = add_user_to_groups(user, &projects) {
                    info!(target: "update", "Failed to add user {} to project groups: {}", user, e);
                }
            }
            Err(e) => {
                error!(target: "update", "Failed to fetch projects for user {}: {}", user, e);
            }
        }
    }
    Ok(())
}
