use std::env;
use std::process::Command;

use nix::unistd::{fork, ForkResult};

use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::notifier;

pub fn handle_su() -> Result<()> {
    let pam_type = env::var("PAM_TYPE")
                     .chain_err(|| "PAM_TYPE not set. If you are running this by `watchdog su`, please don't. It's an internal command, intended to be used by PAM.")?;

    let pam_ruser = env::var("PAM_RUSER")
                      .chain_err(|| "PAM_RUSER not set. If you are running this by `watchdog su`, please don't. It's an internal command, intended to be used by PAM.")?;

    let pam_user = env::var("PAM_USER")
                     .chain_err(|| "PAM_USER not set. If you are running this by `watchdog su`, please don't. It's an internal command, intended to be used by PAM.")?;

    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;

        match fork() {
            Ok(ForkResult::Parent { .. }) => {}
            Ok(ForkResult::Child) => {
                notifier::post_su_summary(&config, pam_ruser, pam_user)?;
            }
            Err(_) => println!("Fork failed"),
        }
    }
    Ok(())
}

pub fn handle_su_logs() {
    Command::new("less")
        .arg("/opt/watchdog/logs/su.logs")
        .status()
        .expect("Something went wrong. Is `less` command present in your environment?");
}
