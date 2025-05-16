extern crate reqwest;
extern crate serde_json;

use crate::config::Config;
use crate::errors::*;
use crate::logger::LogTarget;
use log::info;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde_json::json;

/// Notifier is an abstract trait to post messages to webhook
///
/// This trait can be implemented for various webhook based applications
/// like Slack, Discord etc.
pub trait Notifier {
    /// Returns corresponding `Notifier` from watchdog config
    fn new(conf: &Config) -> Option<Self>
    where
        Self: Sized;
    /// Post summary for sudo attempts
    fn post_sudo_summary(
        &self,
        conf: &Config,
        pam_ruser: String,
        pwd: String,
        cmd: String,
    ) -> Result<()>;
    /// Post summary for su attempts
    fn post_su_summary(&self, conf: &Config, from: String, to: String) -> Result<()>;
    /// Post summary for ssh attempts
    fn post_ssh_summary(
        &self,
        conf: &Config,
        success: bool,
        user: String,
        pam_user: String,
    ) -> Result<()>;
}

struct GlobalNotifier(Vec<Box<dyn Notifier>>);

/// Post summary for sudo attempts
pub fn post_sudo_summary(conf: &Config, pam_ruser: String, pwd: String, cmd: String) -> Result<()> {
    let global_notifier = setup(conf);
    for notif in &global_notifier.0 {
        notif.post_sudo_summary(conf, pam_ruser.clone(), pwd.clone(), cmd.clone())?
    }
    Ok(())
}

/// Post summary for su attempts
pub fn post_su_summary(conf: &Config, from: String, to: String) -> Result<()> {
    let global_notifier = setup(conf);
    for notif in &global_notifier.0 {
        notif.post_su_summary(conf, from.clone(), to.clone())?;
    }
    Ok(())
}

/// Post summary for ssh attempts
pub fn post_ssh_summary(
    conf: &Config,
    success: bool,
    user: &String,
    pam_user: &String,
) -> Result<()> {
    let global_notifier = setup(conf);
    for notif in &global_notifier.0 {
        notif.post_ssh_summary(conf, success, user.clone(), pam_user.clone())?;
    }
    Ok(())
}

fn setup(conf: &Config) -> GlobalNotifier {
    let mut register: Vec<Box<dyn Notifier>> = Vec::new();
    if let Some(slack) = Slack::new(conf) {
        register.push(Box::new(slack));
    }
    GlobalNotifier(register)
}

/// Implements `Notifier` trait for slack
#[derive(Debug)]
pub struct Slack {
    token: String,
    channel: String,
    client: Client,
}

impl Slack {
    fn post_message(&self, text: &str, thread_ts: Option<&str>) -> Result<()> {
        let mut payload = json!({
            "channel": self.channel,
            "text": text
        });
        if let Some(ts) = thread_ts {
            payload["thread_ts"] = json!(ts);
        }
        info!(target: LogTarget::WATCHDOG.as_str(), "Slack payload: {:?}", payload);
        let mut res = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .chain_err(|| "Failed to send Slack message")?;

        let body: serde_json::Value = res.json().chain_err(|| "Invalid JSON from Slack")?;
        if !body["ok"].as_bool().unwrap_or(false) {
            return Err(format!(
                "Slack API error: {}",
                body["error"].as_str().unwrap_or("unknown")
            )
            .into());
        }

        Ok(())
    }

    fn fetch_latest_ts(&self) -> Result<String> {
        let mut res = self
            .client
            .get("https://slack.com/api/conversations.history")
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .query(&[("channel", &self.channel), ("limit", &"1".to_string())])
            .send()
            .chain_err(|| "Failed to fetch message history")?;
        info!(target: LogTarget::WATCHDOG.as_str(), "Slack response: {:?}", res);
        let body: serde_json::Value = res.json().chain_err(|| "Invalid JSON from Slack")?;
        if !body["ok"].as_bool().unwrap_or(false) {
            return Err(format!(
                "Slack history API error: {}",
                body["error"].as_str().unwrap_or("unknown")
            )
            .into());
        }
        info!(target: LogTarget::WATCHDOG.as_str(), "Slack body: {:?}", body);
        let ts = body["messages"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|msg| msg["ts"].as_str())
            .ok_or("No messages found in channel")?;
        info!(target: LogTarget::WATCHDOG.as_str(), "Slack timestamp: {:?}", ts);
        Ok(ts.to_string())
    }
}

impl Notifier for Slack {
    fn new(conf: &Config) -> Option<Slack> {
        let token = conf.notifiers.token.trim();
        let channel = conf.notifiers.channel.trim();

        if token.is_empty() || channel.is_empty() {
            return None;
        }

        Some(Slack {
            token: token.to_string(),
            channel: channel.to_string(),
            client: Client::new(),
        })
    }

    fn post_sudo_summary(
        &self,
        conf: &Config,
        pam_ruser: String,
        pwd: String,
        cmd: String,
    ) -> Result<()> {
        let parent_text = format!("{} attempted sudo on {}", pam_ruser, conf.hostname);
        self.post_message(&parent_text, None)?;
        info!(target: LogTarget::WATCHDOG.as_str(), "Posted parent message: {:?}", parent_text);

        let thread_ts = self.fetch_latest_ts()?;
        info!(target: LogTarget::WATCHDOG.as_str(), "Fetched thread timestamp: {:?}", thread_ts);

        let pwd_text = format!("Attempted in :{} ", pwd);
        self.post_message(&pwd_text, Some(&thread_ts))?;

        let cmd_text = format!("Command :{} ", cmd);
        self.post_message(&cmd_text, Some(&thread_ts))?;

        Ok(())
    }

    fn post_su_summary(&self, conf: &Config, from: String, to: String) -> Result<()> {
        let text = format!(
            "Switched user from *{}* to *{}* on {}",
            from, to, conf.hostname
        );
        self.post_message(&text, None)?;
        Ok(())
    }

    fn post_ssh_summary(
        &self,
        conf: &Config,
        success: bool,
        user: String,
        pam_user: String,
    ) -> Result<()> {
        let text = if success {
            format!("{} logged in on {}@{}", user, pam_user, conf.hostname)
        } else {
            format!("{} tried to log in on {}@{}", user, pam_user, conf.hostname)
        };
        self.post_message(&text, None)?;
        Ok(())
    }
}
