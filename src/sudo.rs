use std::env;
use std::process::Command;

use nix::unistd::{fork, ForkResult};

use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::notifier;
use lib::logger;
use lib::utils::SUDO_LOG_PATH;

pub fn handle_sudo() -> Result<()> {
    let pam_type = env::var("PAM_TYPE")
                     .chain_err(|| "PAM_TYPE not set. If you are running this by `watchdog sudo`, please don't. It's an internal command, intended to be used by PAM.")?;

    let pam_ruser = env::var("PAM_RUSER")
                     .chain_err(|| "PAM_RUSER not set. If you are running this by `watchdog sudo`, please don't. It's an internal command, intended to be used by PAM.")?;
    logger::logln(&format!("PAM_RUSER: {}", pam_ruser));
    logger::logln(&format!("PAM_TYPE: {}", pam_type));
    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;
        if let Err(e) = logger::log(SUDO_LOG_PATH, "SUCCESS", &format!("User: {}", pam_ruser)) {
            println!("Failed to log: {}", e);
        }
        logger::logln("Logging successful");
        match fork() {
            Ok(ForkResult::Parent { .. }) => {}
            Ok(ForkResult::Child) => {
                notifier::post_sudo_summary(&config, pam_ruser)?;
            }
            Err(_) => println!("Fork failed"),
        }
    }

    Ok(())
}

pub fn handle_sudo_logs() {
    Command::new("less")
        .arg("/opt/watchdog/logs/sudo.logs")
        .status()
        .expect("Something went wrong. Is `less` command present in your environment?");
}
