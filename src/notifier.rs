extern crate log;
extern crate reqwest;
extern crate serde_json;

use crate::config;
use log::info;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

/// Notifier is an abstract trait to post messages to webhook
///
/// This trait can be implemented for various webhook based applications
/// like Slack, Discord etc.
pub trait Notifier<'a> {
    /// Returns corresponding `Notifier` from watchdog config
    fn new(conf: &'a config::Config) -> Option<Self>
    where
        Self: Sized;
    /// URL returns the webhook url of the Notifier
    fn url(&self) -> &str;
    /// Send request with message
    fn make_request(&self, json: String);
    /// Post summary for sudo attempts
    fn post_sudo_summary(&self, conf: &config::Config, pam_ruser: String);
    /// Post summary for su attempts
    fn post_su_summary(&self, conf: &config::Config, from: String, to: String);
    /// Post summary for ssh attempts
    fn post_ssh_summary(
        &self,
        conf: &config::Config,
        success: bool,
        user: String,
        pam_ruser: String,
    );
}

/// Implements `Notifier` trait for slack
#[derive(Debug)]
pub struct Slack<'a>(&'a str);

impl Slack<'_> {
    /// Creates JSON to be sent in the make_request
    ///
    /// Takes two arguments: `text` and `color`.
    /// * `text` is the message to be displayed in the message on Slack.
    ///   It accepts markdown format string.
    /// * `color` is a hexcode color string prefixed with `#`.
    ///   It's the color of message accent on Slack.
    fn create_json(text: &str, color: &str) -> String {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        json!({
            "attachments": [
                {
                    "text": format!("{}", text),
                    "mrkdwn_in": ["text"],
                    "ts": format!("{}", since_the_epoch.as_secs()),
                    "color": format!("{}", color)
                }
            ]
        })
        .to_string()
    }
}

impl<'a> Notifier<'a> for Slack<'a> {
    fn new(conf: &'a config::Config) -> Option<Slack> {
        let url: &'a str = conf.slack_api_url.trim();
        if url.len() == 0 {
            return None;
        }
        Some(Slack(url))
    }

    fn url(&self) -> &str {
        self.0
    }

    fn make_request(&self, json: String) {
        let client = reqwest::Client::new();
        let res = client
            .post(self.url())
            .header(CONTENT_TYPE, "application/json")
            .body(json)
            .send();

        match res {
            Ok(_) => {}
            Err(_) => {
                info!("Couldn't make request on slack. Activity couldn't be logged on slack.")
            }
        }
    }

    fn post_sudo_summary(&self, conf: &config::Config, pam_ruser: String) {
        let text = format!("sudo attempted on {}@{}", pam_ruser, conf.keyhouse_hostname);
        let json = Slack::create_json(&text, "#36a64f");
        self.make_request(json);
    }

    fn post_su_summary(&self, conf: &config::Config, from: String, to: String) {
        let text = format!(
            "switched user from {} to {} on {}",
            from, to, conf.keyhouse_hostname
        );
        let json = Slack::create_json(&text, "#36a64f");
        self.make_request(json);
    }

    fn post_ssh_summary(
        &self,
        conf: &config::Config,
        success: bool,
        user: String,
        pam_ruser: String,
    ) {
        let color: &str;
        let text: String;
        if success {
            text = format!(
                "test: {} logged in on {}@{}",
                user, pam_ruser, conf.keyhouse_hostname
            );
            color = "#36a64f";
        } else {
            text = format!(
                "test: {} tried to log in on {}@{}",
                user, pam_ruser, conf.keyhouse_hostname
            );
            color = "#f29513";
        }
        let json = Slack::create_json(&text, color);
        self.make_request(json);
    }
}
