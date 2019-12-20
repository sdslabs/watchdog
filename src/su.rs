use std::env;
use common_lib::notifier::{Notifier, Slack};
use common_lib::config::read_config;
use common_lib::init::init;
use common_lib::errors::*;

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

        match Slack::new(&config) {
            Some(notifier) => notifier.post_su_summary(&config, pam_ruser, pam_user)?,
            None => {}
        };
    }
    Ok(())
}

pub fn handle_su_logs() {
    println!("watchdog-su logs:");
    /* TODO: Filter logs specific to su */
}
