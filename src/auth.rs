use std::fs;
use common_lib::notifier::{Notifier, Slack};
use common_lib::init::init;
use common_lib::config::read_config;
use common_lib::keyhouse::{validate_user, get_name};
use common_lib::errors::*;
use std::process::Command;

pub fn handle_auth(ssh_host_username: &str, ssh_key: &str) -> Result<()> {
    let config = read_config()?;
    init(&config)?;

    match validate_user(&config, ssh_host_username.to_string(), ssh_key) {
        Ok(true) => {
            let data = format!(
                "ssh_host_username = '{}'\nssh_key = '{}'\n",
                ssh_host_username, ssh_key
            );

            fs::write(&config.temp_env_file, data)
                        .chain_err(|| "Cannot write temporary environment file. Please check if the watchdog `auth_keys_cmd` is run by the root user")?;

            println!("{}", ssh_key);
            Ok(())
        }

        Ok(false) => {
            let name = get_name(&config, ssh_key)?;

            match Slack::new(&config) {
                Some(notifier) => {
                    notifier.post_ssh_summary(&config, false, name, ssh_host_username.to_string())?
                }
                None => {}
            };
            Ok(())
        }

        Err(e) => {
            Err(e).chain_err(|| "Error while validating user from keyhouse")
        }
    }
}

pub fn handle_auth_logs() {
    Command::new("less").arg("/opt/watchdog/logs/ssh.logs").status().expect("Something went wrong. Is `less` command present in your environment?");
}
