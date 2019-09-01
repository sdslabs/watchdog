use std::env;

extern crate watchdog;

fn main(){

	let config = watchdog::config::read_config();
	let args: Vec<_> = env::args().collect();

	let ssh_host_username = &args[1];
	let _ssh_host_user_home = &args[2];
	let ssh_key = format!("{} {}", args[3], args[5]);

	if watchdog::keyhouse::validate_user(config.clone(), ssh_host_username.to_string(), ssh_key.clone()) {
		println!("{}", watchdog::keyhouse::get_name(config, ssh_key));
	}

}