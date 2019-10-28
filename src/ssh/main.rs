extern crate watchdog;

use std::env;

fn main() {
    let pam_type = match env::var("PAM_TYPE") {
        Ok(val) => val,
        Err(_e) => String::new(),
    };

    if pam_type == "open_session" {
        let config = watchdog::config::read_config();
        watchdog::init::init(&config);

        let env = watchdog::environment::read_temp_env(&config.temp_env_file);
        let name = watchdog::keyhouse::get_name(&config, env.ssh_key);

        watchdog::slack::post_ssh_summary(&config, true, name, env.ssh_host_username);
        watchdog::utils::clear_file(&config.temp_env_file);
    }
}
