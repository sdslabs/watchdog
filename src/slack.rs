use crate::config;
extern crate reqwest;
use reqwest::header::CONTENT_TYPE;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_request_on_slack(config: config::Config, json: String) {
	let client = reqwest::Client::new();
	let _res = client.post(&config.slack_api_url)
	    .header(CONTENT_TYPE, "application/json")
	    .body(json)
	    .send();
}

fn slack_json (text: String, color: String) -> String {
	let start = SystemTime::now();
	let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
	let json = format!("
	{{
		\"attachments\": [
			{{
				\"text\": \"{}\",
				\"mrkdwn_in\": [\"text\"],
				\"ts\": \"{}\",
				\"color\": \"{}\"
			}}
		]
	}}
	", text, since_the_epoch.as_secs(), color);
	return json;
}

pub fn post_sudo_summary(config: config::Config, pam_ruser: String) {
	let text = format!("sudo attempted on {}@{}", pam_ruser, config.keyhouse_hostname);
	let color = String::from("#36a64f");
	let json = slack_json(text, color);
	make_request_on_slack(config, json);
}
