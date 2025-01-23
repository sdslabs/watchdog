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

    let pam_tty = env::var("PAM_TTY") //gives terminal session
                    .chain_err(|| "PAM_TTY not set. If you are running this by `watchdog ssh`, please don't. It's an internal command, intended to be used by PAM.")?;

    let pam_ruser = env::var("PAM_RUSER") //gives ssh_host_username
                    .chain_err(|| "PAM_RUSER not set. If you are running this by `watchdog ssh`, please don't. It's an internal command, intended to be used by PAM.")?;

    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;
        
        let file_name = pam_tty.replace("/", "_"); //dev_pts_0

        let env = read_temp_env("/opt/watchdog/ssh_env/file_name")?; //read appropriate env file
        let name = get_name(&config, &env.ssh_key)?;

        match fork() {
            Ok(ForkResult::Parent { .. }) => {
                clear_file("/opt/watchdog/ssh_env/file_name")?;
            }
            Ok(ForkResult::Child) => {
                notifier::post_ssh_summary(&config, true, name, pam_ruser)?;
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
