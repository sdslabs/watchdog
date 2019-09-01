use std::env;
extern crate reqwest;
extern crate crypto;
use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;

extern crate watchdog;

fn validate_user(config: watchdog::config::Config, user: String, ssh_key: String) -> bool {
	let mut hasher = Sha256::new();
	hasher.input_str(&ssh_key);
	let hex = hasher.result_str();
	let res = reqwest::get(&format!("{}/{}/{}/{}/{}", config.keyhouse_base_url, config.keyhouse_token, config.keyhouse_hostname, user, hex));
	if res.unwrap().status().is_success() {
		return true;
	}
	else {
		return false;
	}
}

fn main(){

	let config = watchdog::config::read_config();
	let args: Vec<_> = env::args().collect();

	let ssh_host_username = &args[1];
	let ssh_host_user_home = &args[2];
	let ssh_key = format!("{} {}", args[3], args[5]);

	if validate_user(config, ssh_host_username.to_string(), ssh_key) {
		println!("Yo");
	}

}