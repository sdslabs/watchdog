use std::env;
use std::process::Command;

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
    let pam_user= env::var("PAM_USER")
                    .chain_err(|| "PAM_USER not set. If you are running this by `watchdog ssh`, please don't. It's an internal command, intended to be used by PAM.")?;
    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;

        if let Err(e) = logger::log(SSH_LOG_PATH, "SUCCESS", &format!("User: {}", pam_user)) {
            println!("Failed to log: {}", e);
        }
        logger::logln("Logging successful");
        match fork() {
            Ok(ForkResult::Parent { .. }) => {}
            Ok(ForkResult::Child) => {
                notifier::post_ssh_summary(&config, true, &pam_user,&pam_user)?;
            }
            Err(_) => println!("Fork failed"),
        }

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


