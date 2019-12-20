use std::env;
use common_lib::notifier::{Notifier, Slack};
use common_lib::config::read_config;
use common_lib::init::init;
use common_lib::errors::*;

pub fn handle_sudo() -> Result<()>{
    let pam_type = env::var("PAM_TYPE")
                     .chain_err(|| "PAM_TYPE not set. If you are running this by `watchdog sudo`, please don't. It's an internal command, intended to be used by PAM.")?;

    let pam_ruser = env::var("PAM_RUSER")
                      .chain_err(|| "PAM_TYPE not set. If you are running this by `watchdog sudo`, please don't. It's an internal command, intended to be used by PAM.")?;

    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;

        match Slack::new(&config) {
            Some(notifier) => notifier.post_sudo_summary(&config, pam_ruser),
            None => {}
        };
    }

    Ok(())
}

pub fn handle_sudo_logs() {
    println!("watchdog-sudo logs:");
    /* TODO: Filter logs specific to sudo */
}