use std::env;
use common_lib::notifier::{Notifier, Slack};
use common_lib::config::read_config;
use common_lib::init::init;

pub fn handle_su() {
    let pam_type = match env::var("PAM_TYPE") {
        Ok(val) => val,
        Err(_e) => String::new(),
    };
    let pam_ruser = match env::var("PAM_RUSER") {
        Ok(val) => val,
        Err(_e) => String::new(),
    };
    let pam_user = match env::var("PAM_USER") {
        Ok(val) => val,
        Err(_e) => String::new(),
    };

    if pam_type == "open_session" {
        let config = read_config();
        init(&config);

        match Slack::new(&config) {
            Some(notifier) => notifier.post_su_summary(&config, pam_ruser, pam_user),
            None => {}
        };
    }
}

pub fn handle_su_logs() {
    println!("watchdog-su logs:");
    /* TODO: Filter logs specific to su */
}
