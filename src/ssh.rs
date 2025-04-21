use std::env;

use lib::errors::*;
use lib::init::init;
use lib::notifier;
use lib::{config::read_config, logger::LogTarget};
use log::{error, info};
use nix::unistd::{fork, ForkResult};

pub fn handle_ssh() -> Result<()> {
    info!(target: LogTarget::SSH.as_str(), "in handle_ssh SSH Command");
    let pam_type = env::var("PAM_TYPE")
                    .chain_err(|| "PAM_TYPE not set. If you are running this by `watchdog ssh`, please don't. It's an internal command, intended to be used by PAM.")?;
    info!(target: LogTarget::SSH.as_str(), "PAM_TYPE: {}", pam_type);
    let pam_user= env::var("PAM_USER")
                    .chain_err(|| "PAM_USER not set. If you are running this by `watchdog ssh`, please don't. It's an internal command, intended to be used by PAM.")?;
    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;
        info!(target: LogTarget::SSH.as_str(), "Logging successful");
        match fork() {
            Ok(ForkResult::Parent { .. }) => {}
            Ok(ForkResult::Child) => {
                notifier::post_ssh_summary(&config, true, &pam_user, &pam_user)?;
            }
            Err(_) => error!("Fork failed"),
        }
    }
    Ok(())
}
