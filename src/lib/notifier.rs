extern crate reqwest;
extern crate serde_json;

use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::config::Config;
use crate::errors::*;

/// Notifier is an abstract trait to post messages to webhook
///
/// This trait can be implemented for various webhook based applications
/// like Slack, Discord etc.
pub trait Notifier {
    /// Returns corresponding `Notifier` from watchdog config
    fn new(conf: &Config) -> Option<Self>
    where
        Self: Sized;
    /// URL returns the webhook url of the Notifier
    fn url(&self) -> &str;
    /// Send request with message
    fn make_request(&self, json: String) -> Result<()>;
    /// Post summary for sudo attempts
    fn post_sudo_summary(&self, conf: &Config, pam_ruser: String) -> Result<()>;
    /// Post summary for su attempts
    fn post_su_summary(&self, conf: &Config, from: String, to: String) -> Result<()>;
    /// Post summary for ssh attempts
    fn post_ssh_summary(
        &self,
        conf: &Config,
        success: bool,
        user: String,
        pam_ruser: String,
    ) -> Result<()>;
}

struct GlobalNotifier(Vec<Box<dyn Notifier>>);

/// Post summary for sudo attempts
pub fn post_sudo_summary(conf: &Config, pam_ruser: String) -> Result<()> {
    let global_notifier = setup(conf);
    for notif in &global_notifier.0 {
        let pam_ruser_copy = String::from(&pam_ruser);
        notif.post_sudo_summary(conf, pam_ruser_copy)?
    }
    Ok(())
}

/// Post summary for su attempts
pub fn post_su_summary(conf: &Config, from: String, to: String) -> Result<()> {
    let global_notifier = setup(conf);
    for notif in &global_notifier.0 {
        let from_copy = String::from(&from);
        let to_copy = String::from(&to);
        notif.post_su_summary(conf, from_copy, to_copy)?;
    }
    Ok(())
}

/// Post summary for ssh attempts
pub fn post_ssh_summary(
    conf: &Config,
    success: bool,
    user: String,
    pam_ruser: String,
) -> Result<()> {
    let global_notifier = setup(conf);
    for notif in &global_notifier.0 {
        let user_copy = String::from(&user);
        let pam_ruser_copy = String::from(&pam_ruser);
        notif.post_ssh_summary(conf, success, user_copy, pam_ruser_copy)?;
    }
    Ok(())
}

fn setup(conf: &Config) -> GlobalNotifier {
    let mut register: Vec<Box<dyn Notifier>> = Vec::new();
    match Slack::new(conf) {
        Some(slack) => {
            register.push(Box::new(slack));
        }
        None => {}
    };
    GlobalNotifier(register)
}

/// Implements `Notifier` trait for slack
#[derive(Debug)]
pub struct Slack(String);

impl Slack {
    /// Creates JSON to be sent in the make_request
    ///
    /// Takes two arguments: `text` and `color`.
    /// * `text` is the message to be displayed in the message on Slack.
    ///   It accepts markdown format string.
    /// * `color` is a hexcode color string prefixed with `#`.
    ///   It's the color of message accent on Slack.
    fn create_json(text: &str, color: &str) -> Result<String> {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)?;
        let json_text = json!({
            "attachments": [
                {
                    "text": format!("{}", text),
                    "mrkdwn_in": ["text"],
                    "ts": format!("{}", since_the_epoch.as_secs()),
                    "color": format!("{}", color)
                }
            ]
        })
        .to_string();
        Ok(json_text)
    }
}

impl Notifier for Slack {
    fn new(conf: &Config) -> Option<Slack> {
        let url: &str = conf.notifiers.slack.trim();
        if url.len() == 0 {
            return None;
        }
        Some(Slack(String::from(url)))
    }

    fn url(&self) -> &str {
        &self.0
    }

    fn make_request(&self, json: String) -> Result<()> {
        let client = reqwest::Client::new();
        let res = client
            .post(self.url())
            .header(CONTENT_TYPE, "application/json")
            .body(json)
            .send();

        res.chain_err(|| "Error while creating a request to Slack Webhook")?;
        Ok(())
    }

    fn post_sudo_summary(&self, conf: &Config, pam_ruser: String) -> Result<()> {
        let text = format!("sudo attempted on {}@{}", pam_ruser, conf.hostname);
        let json = Slack::create_json(&text, "#36a64f")?;
        self.make_request(json)
            .chain_err(|| "Couldn't post sudo summary to Slack")?;
        Ok(())
    }

    fn post_su_summary(&self, conf: &Config, from: String, to: String) -> Result<()> {
        let text = format!("switched user from {} to {} on {}", from, to, conf.hostname);
        let json = Slack::create_json(&text, "#36a64f")?;
        self.make_request(json)
            .chain_err(|| "Couldn't post su summary to Slack")?;
        Ok(())
    }

    fn post_ssh_summary(
        &self,
        conf: &Config,
        success: bool,
        user: String,
        pam_ruser: String,
    ) -> Result<()> {
        let color: &str;
        let text: String;
        if success {
            text = format!("{} logged in on {}@{}", user, pam_ruser, conf.hostname);
            color = "#36a64f";
        } else {
            text = format!(
                "{} tried to log in on {}@{}",
                user, pam_ruser, conf.hostname
            );
            color = "#f29513";
        }
        let json = Slack::create_json(&text, color)?;
        self.make_request(json)
            .chain_err(|| "Couldn't post ssh summary to Slack")?;
        Ok(())
    }
}
