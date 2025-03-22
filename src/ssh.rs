use std::env;
use std::process::Command;

use lib::keyhouse::fetch_github_projects;
use lib::utils::add_user_to_groups;
use nix::unistd::{fork, ForkResult};

use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::notifier;
//use lib::utils::clear_file;
use lib::logger;
use lib::utils::SSH_LOG_PATH;

pub fn handle_ssh() -> Result<()> {
    logger::logln("in handle_ssh SSH Command");
    let pam_type = env::var("PAM_TYPE")
                    .chain_err(|| "PAM_TYPE not set. If you are running this by `watchdog ssh`, please don't. It's an internal command, intended to be used by PAM.")?;
    logger::logln(&format!("PAM_TYPE: {}", pam_type));
    let pam_ruser= env::var("PAM_RUSER")
                    .chain_err(|| "PAM_RUSER not set. If you are running this by `watchdog ssh`, please don't. It's an internal command, intended to be used by PAM.")?;
    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;

        if let Err(e) = logger::log(SSH_LOG_PATH, "SUCCESS", &format!("User: {}", pam_ruser)) {
            println!("Failed to log: {}", e);
        }
        logger::logln("Logging successful");
        match fork() {
            Ok(ForkResult::Parent { .. }) => {}
            Ok(ForkResult::Child) => {
                notifier::post_ssh_summary(&config, true, &pam_ruser,&pam_ruser)?;
            }
            Err(_) => println!("Fork failed"),
        }

        let mut projects = fetch_github_projects(&config, &pam_ruser)?;

        logger::logln(&format!("Fetched projects: {:?}", projects));

        projects.retain(|p| p != &config.hostname);

        logger::logln(&format!("Filtered projects (excluding self): {:?}", projects));

        match add_user_to_groups(&pam_ruser, &projects) {
            Ok(_) => {}
            Err(e) => {
                logger::logln(&format!("Failed to add user to project groups: {}", e));
            }
        }

        logger::logln("User successfully added to project groups.");
    }
    Ok(())
}

pub fn handle_ssh_logs() {
    logger::logln("in handle_ssh_logs");
    Command::new("less")
        .arg("/opt/watchdog/logs/ssh.logs")
        .status()
        .expect("Something went wrong. Is `less` command present in your environment?");
}


