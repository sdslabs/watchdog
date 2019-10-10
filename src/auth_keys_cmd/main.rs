use std::env;
use std::fs;
// use std::io::prelude::*;

extern crate watchdog;

fn main() {
    let config = watchdog::config::read_config();
    let args: Vec<_> = env::args().collect();

    let ssh_host_username = &args[1];
    let ssh_key = format!("{} {}", args[3], args[5]);

    if watchdog::keyhouse::validate_user(&config, ssh_host_username.to_string(), &ssh_key) {
        let data = format!(
            "ssh_host_username = '{}'\nssh_key = '{}'\n",
            ssh_host_username, ssh_key
        );

        fs::write(&config.temp_env_file, data).expect("unable to write temp env file");

        println!("{}", ssh_key);
    } else {
        let name = watchdog::keyhouse::get_name(&config, ssh_key);
        watchdog::slack::post_ssh_summary(&config, false, name, ssh_host_username.to_string());
    }
}
