use std::env;
use std::process::Command;

use nix::unistd::{fork, ForkResult};

use lib::config::read_config;
use lib::environment::read_temp_env;
use lib::errors::*;
use lib::init::init;
use lib::keyhouse::get_name;
use lib::notifier;
use lib::utils::clear_file;

pub fn handle_ssh() -> Result<()> {
    let pam_type = env::var("PAM_TYPE")
                    .chain_err(|| "PAM_TYPE not set. If you are running this by `watchdog ssh`, please don't. It's an internal command, intended to be used by PAM.")?;

    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;

        let machine_username = env::var("PAM_USER")
            .chain_err(|| "PAM_USER not set. This should be the username of the ssh user")?;
        println!("Machine username: {}", machine_username); //I hope this is the username of the machine

        // let env = read_temp_env("/opt/watchdog/ssh_env")?;
        // let name = get_name(&config, &env.ssh_key)?;
        let ssh_env_file = format!("/opt/watchdog/{}_ssh_env", machine_username); //machine_ssh_env
        let env = read_temp_env(&ssh_env_file)?;
        let name = get_name(&config, &env.ssh_key)?;

        match fork() {
            Ok(ForkResult::Parent { .. }) => {
                clear_file(&ssh_env_file)?;
            }
            Ok(ForkResult::Child) => {
                notifier::post_ssh_summary(&config, true, name, env.ssh_host_username)?;
            }
            Err(_) => println!("Fork failed"),
        }
    }
    Ok(())
}

pub fn handle_ssh_logs() {
    Command::new("less")
        .arg("/opt/watchdog/logs/ssh.logs")
        .status()
        .expect("Something went wrong. Is `less` command present in your environment?");
}
