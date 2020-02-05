use std::env;
use common_lib::notifier::{Notifier, Slack};
use common_lib::init::init;
use common_lib::config::read_config;
use common_lib::environment::read_temp_env;
use common_lib::keyhouse::get_name;
use common_lib::utils::clear_file;
use common_lib::errors::*;
use std::process::Command;

pub fn handle_ssh() -> Result<()> {
    let pam_type = env::var("PAM_TYPE")
                    .chain_err(|| "PAM_TYPE not set. If you are running this by `watchdog ssh`, please don't. It's an internal command, intended to be used by PAM.")?;

    if pam_type == "open_session" {
        let config = read_config()?;
        init(&config)?;

        let env = read_temp_env(&config.temp_env_file)?;
        let name = get_name(&config, &env.ssh_key)?;

        match Slack::new(&config) {
            Some(notifier) => notifier.post_ssh_summary(&config, true, name, env.ssh_host_username)?,
            None => {}
        };

        clear_file(&config.temp_env_file)?;
    }
    Ok(())
}

pub fn handle_ssh_logs() {
    Command::new("less").arg("/opt/watchdog/logs/ssh.logs").status().expect("Something went wrong. Is `less` command present in your environment?");
}
