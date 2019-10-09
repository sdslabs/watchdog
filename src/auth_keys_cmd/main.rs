use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;

extern crate watchdog;

fn main(){

	let config = watchdog::config::read_config();
	let args: Vec<_> = env::args().collect();

	let ssh_host_username = &args[1];
	let _ssh_host_user_home = &args[2];
	let ssh_key = format!("{} {}", args[3], args[5]);

	if watchdog::keyhouse::validate_user(config.clone(), ssh_host_username.to_string(), ssh_key.clone()) {
		let mut file = OpenOptions::new()
			.write(true)
			.open(&config.temp_env_file)
			.unwrap();
		writeln!(file, "ssh_host_username = '{}'", ssh_host_username);
		writeln!(file, "ssh_key = '{}'", ssh_key);

		println!("{}", ssh_key);
		// let name = watchdog::keyhouse::get_name(config.clone(), ssh_key.clone());
		// watchdog::slack::post_ssh_summary(config, true, name, ssh_host_username.to_string());
	}
	else {
		let name = watchdog::keyhouse::get_name(config.clone(), ssh_key);
		watchdog::slack::post_ssh_summary(config, false, name, ssh_host_username.to_string());
	}

}