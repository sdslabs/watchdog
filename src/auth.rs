use log::{error, info};
use nix::unistd::{fork, ForkResult};

use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::keyhouse::{get_name, validate_user};
use lib::notifier;

pub fn handle_auth(ssh_host_username: &str, ssh_key: &str) -> Result<()> {
    let config = read_config()?;
    init(&config)?;
    info!(target: "auth", "ssh_key in handle_auth: {}", ssh_key);
    match validate_user(&config, ssh_host_username.to_string(), ssh_key) {
        Ok(true) => {
            info!(target: "auth", "User validated by handle auth");
            println!("{}", ssh_key);
            Ok(())
        }

        Ok(false) => {
            info!(target: "auth", "User not validated");
            let name = get_name(&config, ssh_key)?;
            info!(target: "auth", "Logging failed");
            match fork() {
                Ok(ForkResult::Parent { .. }) => {}
                Ok(ForkResult::Child) => {
                    notifier::post_ssh_summary(
                        &config,
                        false,
                        &name,
                        &ssh_host_username.to_string(),
                    )?;
                    std::process::exit(0);
                }
                Err(_) => println!("Fork failed"),
            }
            Ok(())
        }
        Err(e) => {
            error!(target: "auth", "Error while validating user from keyhouse");
            Err(e).chain_err(|| "Error while validating user from keyhouse")
        }
    }
}
