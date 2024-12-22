use std::fs;

use nix::unistd::{fork, ForkResult};

use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::keyhouse::{get_name, validate_user};
use lib::notifier;
use lib::logger;
use lib::utils::AUTH_LOG_PATH;

pub fn handle_auth(ssh_host_username: &str, ssh_key: &str) -> Result<()> {
    let config = read_config()?;
    init(&config)?;
    logger::logln(&format!("ssh_key in handle_auth: {}", ssh_key));
    match validate_user(&config, ssh_host_username.to_string(), ssh_key) {
        Ok(true) => {
            logger::logln("User validated");
            let data = format!(
                "ssh_host_username = '{}'\nssh_key = '{}'\n",
                ssh_host_username, ssh_key
            );

            fs::write("/opt/watchdog/ssh_env", data)
                        .chain_err(|| "Cannot write temporary environment file. Please check if the watchdog `auth_keys_cmd` is run by the root user")?;
            logger::logln("Temporary environment file written");
            println!("{}", ssh_key);
            let name = get_name(&config, ssh_key)?;
            if let Err(e) = logger::log(AUTH_LOG_PATH, "SUCCESS", &format!("User: {}", name)) {
                println!("Failed to log: {}", e);
            }
            logger::logln("Logging successful");
            Ok(())
        }

        Ok(false) => {
            logger::logln("User not validated");
            let name = get_name(&config, ssh_key)?;
            if let Err(e) = logger::log(AUTH_LOG_PATH, "Failed", &format!("User: {}", name)) {
                println!("Failed to log: {}", e);
            }
            logger::logln("Logging failed");
            match fork() {
                Ok(ForkResult::Parent { .. }) => {}
                Ok(ForkResult::Child) => {
                    notifier::post_ssh_summary(
                        &config,
                        false,
                        name,
                        ssh_host_username.to_string(),
                    )?;
                    std::process::exit(0); 
                }
                Err(_) => println!("Fork failed"),
            }
            Ok(())
        }
        Err(e) => {
            logger::logln("Error while validating user from keyhouse");
            Err(e).chain_err(|| "Error while validating user from keyhouse")
        }
    }
}
