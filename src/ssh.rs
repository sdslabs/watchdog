use std::env;
use std::process::Command;

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

        let env = read_temp_env("/opt/watchdog/ssh_env")?;
        let name = get_name(&config, &env.ssh_key)?;

        notifier::post_ssh_summary(&config, true, name, env.ssh_host_username)?;

        clear_file("/opt/watchdog/ssh_env")?;
    }
    Ok(())
}

pub fn handle_ssh_logs() {
    Command::new("less")
        .arg("/opt/watchdog/logs/ssh.logs")
        .status()
        .expect("Something went wrong. Is `less` command present in your environment?");
}
