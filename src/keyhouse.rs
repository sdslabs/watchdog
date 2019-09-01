use crate::config;
extern crate reqwest;
extern crate crypto;
use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;

pub fn validate_user(config: config::Config, user: String, ssh_key: String) -> bool {
	let mut hasher = Sha256::new();
	hasher.input_str(&ssh_key);
	let hex = hasher.result_str();
	let res = reqwest::get(&format!("{}/{}/access/{}/{}/{}", config.keyhouse_base_url, config.keyhouse_token, config.keyhouse_hostname, user, hex));
	if res.unwrap().status().is_success() {
		return true;
	}
	else {
		return false;
	}
}

pub fn get_name(config: config::Config, ssh_key: String) -> String {
	let mut hasher = Sha256::new();
	hasher.input_str(&ssh_key);
	let hex = hasher.result_str();
	let mut res = reqwest::get(&format!("{}/{}/names/{}", config.keyhouse_base_url, config.keyhouse_token, hex)).unwrap();
	if res.status().is_success() {
		let name = res.text().unwrap();
		return name;
	}
	else {
		return String::new();
	}
}