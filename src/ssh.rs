use std::env;
use common_lib::notifier::{Notifier, Slack};
use common_lib::init::init;
use common_lib::config::read_config;
use common_lib::environment::read_temp_env;
use common_lib::keyhouse::get_name;
use common_lib::utils::clear_file;

pub fn handle_ssh() {
    let pam_type = match env::var("PAM_TYPE") {
        Ok(val) => val,
        Err(_e) => String::new(),
    };

    if pam_type == "open_session" {
        let config = read_config();
        init(&config);

        let env = read_temp_env(&config.temp_env_file);
        let name = get_name(&config, env.ssh_key);

        match Slack::new(&config) {
            Some(notifier) => notifier.post_ssh_summary(&config, true, name, env.ssh_host_username),
            None => {}
        };

        clear_file(&config.temp_env_file);
    }
}

pub fn handle_ssh_logs() {
    println!("watchdog-ssh logs:");
    /* TODO: Filter logs specific to ssh */
}
