use std::fs;
use std::env;

use nix::unistd::{fork, ForkResult};

use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::keyhouse::{get_name, validate_user};
use lib::notifier;

pub fn handle_auth(ssh_host_username: &str, ssh_key: &str) -> Result<()> {
    let config = read_config()?;
    init(&config)?;

    let pam_tty = env::var("PAM_TTY") //gives terminal session
                     .chain_err(|| "PAM_TTY not set. If you are running this by `watchdog sudo`, please don't. It's an internal command, intended to be used by PAM.")?;

    match validate_user(&config, ssh_host_username.to_string(), ssh_key) {
        Ok(true) => {
            let data = format!(
                "ssh_key = '{}'\n",
                ssh_key
            );

            //file name is pam_tty
            let file_name = pam_tty.replace("/", "_");

            let path = format!("/opt/watchdog/ssh_env/{}", file_name);

            fs::write(&path, data)
                        .chain_err(|| "Cannot write temporary environment file. Please check if the watchdog `auth_keys_cmd` is run by the root user")?;

            println!("{}", ssh_key);
            Ok(())
        }

        Ok(false) => {
            let name = get_name(&config, ssh_key)?;

            match fork() {
                Ok(ForkResult::Parent { .. }) => {}
                Ok(ForkResult::Child) => {
                    notifier::post_ssh_summary(
                        &config,
                        false,
                        name,
                        ssh_host_username.to_string(),
                    )?;
                }
                Err(_) => println!("Fork failed"),
            }
            Ok(())
        }

        Err(e) => Err(e).chain_err(|| "Error while validating user from keyhouse"),
    }
}
