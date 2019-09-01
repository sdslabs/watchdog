use std::env;

extern crate watchdog;

fn main(){

	let pam_type = match env::var("PAM_TYPE") {
		Ok(val) => val,
		Err(_e) => String::new(),
	};
	let pam_ruser = match env::var("PAM_RUSER") {
		Ok(val) => val,
		Err(_e) => String::new(),
	};
	let pam_tty = match env::var("PAM_TTY") {
		Ok(val) => val,
		Err(_e) => String::new(),
	};

	if pam_type == "close_session" {
		let config = watchdog::config::read_config();
		watchdog::slack::post_sudo_summary(config, pam_ruser);
	}

}