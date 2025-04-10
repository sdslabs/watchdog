
use lib::keyhouse::fetch_file_names;
use lib::keyhouse::fetch_github_projects;
use lib::utils::add_user_to_groups;

use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::logger;

pub fn handle_update() -> Result<()> {
    let mut users: Vec<String>=  Vec::new();
    logger::logln("in handle_update");
    let config= read_config()?;
    init(&config)?;
    let _ = fetch_file_names(&config.keyhouse.base_url, "data/hosts", &config.keyhouse.token, &mut users)?;
    logger::logln(&format!("Fetched users: {:?}", users));
    println!("Fetched users: {:?}", users);
    for user in users.iter() {
        match fetch_github_projects(&config, user) {
            Ok(mut projects) => {
                logger::logln(&format!("Fetched projects: {:?}", projects));
                projects.retain(|p| p != &config.hostname);
                logger::logln(&format!(
                    "Filtered projects (excluding self): {:?}",
                    projects
                ));
                if let Err(e) = add_user_to_groups(user, &projects) {
                    logger::logln(&format!(
                        "Failed to add user {} to project groups: {}",
                        user, e
                    ));
                }
            }
            Err(e) => {
                logger::logln(&format!(
                    "Failed to fetch projects for user {}: {}",
                    user, e
                ));
            }
        }
    }
   Ok(())
}
