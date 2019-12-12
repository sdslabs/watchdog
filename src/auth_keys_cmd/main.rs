extern crate log;
extern crate watchdog;

use log::error;
use std::env;
use std::fs;
use watchdog::notifier::{Notifier, Slack};

fn main() {
    let config = watchdog::config::read_config();
    watchdog::init::init(&config);

    let args: Vec<_> = env::args().collect();

    let ssh_host_username = &args[1];
    let ssh_key = format!("{} {}", args[3], args[5]);

    if watchdog::keyhouse::validate_user(&config, ssh_host_username.to_string(), &ssh_key) {
        let data = format!(
            "ssh_host_username = '{}'\nssh_key = '{}'\n",
            ssh_host_username, ssh_key
        );

        let res = fs::write(&config.temp_env_file, data);
        match res {
            Ok(b) => b,
            Err(_) => {
                error!("Cannot write temporary environment file. Please check if the watchdog `auth_keys_cmd` is run by the root user");
                std::process::exit(1);
            }
        }

        println!("{}", ssh_key);
    } else {
        let name = watchdog::keyhouse::get_name(&config, ssh_key);

        match Slack::new(&config) {
            Some(notifier) => {
                notifier.post_ssh_summary(&config, false, name, ssh_host_username.to_string())
            }
            None => {}
        };
    }
}
