use std::env;

use lib::{config::read_config, logger::LogTarget};
use lib::errors::*;
use lib::init::init;
use lib::notifier;
use log::{error, info};
use nix::unistd::{fork, ForkResult};

pub fn handle_sudo() -> Result<()> {
    info!(target: LogTarget::SUDO.as_str(), "Handling sudo command");
    let pam_type = env::var("PAM_TYPE")
                     .chain_err(|| "PAM_TYPE not set. If you are running this by `watchdog sudo`, please don't. It's an internal command, intended to be used by PAM.")?;

    let pam_ruser = env::var("PAM_RUSER")
                     .chain_err(|| "PAM_RUSER not set. If you are running this by `watchdog sudo`, please don't. It's an internal command, intended to be used by PAM.")?;
    info!(target: LogTarget::SUDO.as_str(), "PAM_RUSER: {}", pam_ruser);
    info!(target: LogTarget::SUDO.as_str(), "PAM_TYPE: {}", pam_type);
    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;
        info!(target: LogTarget::SUDO.as_str(), "Logging successful");
        match fork() {
            Ok(ForkResult::Parent { .. }) => {}
            Ok(ForkResult::Child) => {
                let pwd = env::var("PWD").unwrap_or_else(|_| {
                    std::env::current_dir()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| "<unknown>".to_string())
                });
                info!(target: LogTarget::SUDO.as_str(), "PWD: {}", pwd);
                notifier::post_sudo_summary(&config, pam_ruser, pwd)?;
            }
            Err(_) => error!("Fork failed"),
        }
    }

    Ok(())
}
