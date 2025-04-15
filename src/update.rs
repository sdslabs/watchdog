use std::process::Command;

use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::keyhouse::fetch_file_names;
use lib::keyhouse::fetch_github_projects;
use lib::utils::add_user_to_groups;
use lib::utils::create_linux_user;
use log::{error, info};

pub fn handle_update() -> Result<()> {
    let mut users: Vec<String> = Vec::new();
    log::info!(target: "update", "Watchdog update triggered.");
    let config = read_config()?;
    init(&config)?;
    let _ = fetch_file_names(
        &config.keyhouse.base_url,
        "data/hosts",
        &config.keyhouse.token,
        &mut users,
    )?;
    info!(target: "update", "Fetched users: {:?}", users);
    println!("Fetched users: {:?}", users);
    for user in users.iter() {
        match create_linux_user(user) {
            Ok(_) => {
                info!(target: "update", "User {} created successfully.", user);
            }
            Err(e) => {
                error!(target: "update", "Failed to create user {}: {}", user, e);
            }
        }
        match fetch_github_projects(&config, user) {
            Ok(mut projects) => {
                info!(target: "update", "Fetched projects for {}: {:?}",user, projects);
                projects.retain(|p| p != &config.hostname);
                info!(target: "update","Filtered projects (excluding self): {:?}", projects);
                if let Err(e) = add_user_to_groups(user, &projects) {
                    info!(target: "update","Failed to add user {} to project groups: {}", user, e);
                }
            }
            Err(e) => {
                error!(target: "update", "Failed to fetch projects for user {}: {}", user, e);
            }
        }
    }
    Ok(())
}
