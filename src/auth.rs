use lib::config::read_config;
use lib::errors::{* , Error};
use lib::init::init;
use lib::keyhouse::{get_name, validate_user};
use lib::logger::LogTarget;
use lib::notifier;
use lib::utils::check_local_cache;
use log::{info};
use nix::unistd::{fork, ForkResult};

#[cfg(feature = "auto-update")]
use crate::update::handle_update;

pub fn handle_auth(ssh_host_username: &str, ssh_key: &str) -> Result<()> {
    let config = read_config()?;
    init(&config)?;
    info!(target: LogTarget::AUTH.as_str(), "ssh_key in handle_auth: {}", ssh_key);

    #[cfg(feature = "auto-update")]
    {
        match handle_update() {
            Ok(_) => {
                info!(target: LogTarget::UPDATE.as_str(), "Update handled successfully");
            }
            Err(e) => {
                error!(target: LogTarget::UPDATE.as_str(), "Error handling update: {}", e);
                return Err(e);
            }
        }
    }

    match validate_user(&config, ssh_host_username.to_string(), ssh_key) {
        Ok(true) => {
            info!(target: LogTarget::AUTH.as_str(), "User validated by handle auth");
            println!("{ssh_key}");
            Ok(())
        }
        Ok(false) | Err(_) => {
            // Try local cache first on validation failure or mismatch
            if check_local_cache(&config, ssh_host_username, ssh_key) {
                info!(target: LogTarget::AUTH.as_str(), "User validated from local cache");
                println!("{ssh_key}");
                Ok(())
            } else {
                info!(target: LogTarget::AUTH.as_str(), "User not validated");
                let name = get_name(&config, ssh_key)?;
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
                Err(Error::from("User validation failed and not found in cache"))
            }
        }
    }
}
