extern crate log;

use log::error;
use std::env;
use std::fs;
use common_lib::notifier::{Notifier, Slack};
use common_lib::init::init;
use common_lib::config::read_config;
use common_lib::keyhouse::{validate_user, get_name};
use common_lib::errors::*;

pub fn handle_auth() -> Result<()> {
    let config = read_config()?;
    init(&config)?;

    let args: Vec<_> = env::args().collect();

    let ssh_host_username = &args[1];
    let ssh_key = format!("{} {}", args[3], args[5]);

    match validate_user(&config, ssh_host_username.to_string(), &ssh_key) {
        Ok(true) => {
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
            Ok(())
        }

        Ok(false) => {
            let name = get_name(&config, ssh_key)?;

            match Slack::new(&config) {
                Some(notifier) => {
                    notifier.post_ssh_summary(&config, false, name, ssh_host_username.to_string())
                }
                None => {}
            };
            Ok(())
        }

        Err(e) => {
            Err(e)
        }
    }
}

pub fn handle_auth_logs() {
    println!("watchdog-auth logs:");
    /* TODO: Filter logs specific to auth */
}
