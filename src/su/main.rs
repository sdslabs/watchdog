extern crate watchdog;

use std::env;
use watchdog::notifier::{Notifier, Slack};

fn main() {
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
        let config = watchdog::config::read_config();
        watchdog::init::init(&config);

        match Slack::new(&config) {
            Some(notifier) => notifier.post_su_summary(&config, pam_ruser, pam_user),
            None => {}
        };
    }
}
