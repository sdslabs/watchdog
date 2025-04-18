use crate::errors::*;
use chrono::FixedOffset;
use log::info;
use std::{fs, process::Command};
pub const AUTH_LOG_PATH: &str = "/opt/watchdog/custom-logs/auth.logs";
pub const SSH_LOG_PATH: &str = "/opt/watchdog/custom-logs/ssh.logs";
pub const SUDO_LOG_PATH: &str = "/opt/watchdog/custom-logs/sudo.logs";
pub const SU_LOG_PATH: &str = "/opt/watchdog/custom-logs/su.logs";

pub fn clear_file(path: &str) -> Result<()> {
    fs::write(path, "")?;
    Ok(())
}

pub fn add_user_to_groups(user: &str, groups: &[String]) -> Result<()> {
    for group in groups {
        if group != user {
            Command::new("usermod")
                .arg("-aG")
                .arg(group)
                .arg(user)
                .output()
                .chain_err(|| format!("Failed to add user {} to group {}", user, group))?;
            info!(target: "update", "User {} added to group {}", user, group);
        }
    }
    Ok(())
}

pub fn create_linux_user(username: &str) -> Result<()> {
    Command::new("useradd")
        .arg("-m")
        .arg("-d")
        .arg("/home")
        .args(&["-s", "/bin/bash"])
        .arg(username)
        .status()
        .chain_err(|| format!("Failed to add user {}", username))?;
    info!(target: "update", "User {} added", username);
    Ok(())
}

pub fn parse_offset(offset_str: &str) -> Result<FixedOffset> {
    let sign = if offset_str.starts_with('+') { 1 } else { -1 };
    let parts: Vec<&str> = offset_str
        .trim_start_matches(&['+', '-'][..])
        .split(':')
        .collect();

    if parts.len() != 2 {
        return Err("Invalid offset format".into());
    }

    let hours: i32 = parts[0].parse().map_err(|_| "Invalid hour format")?;
    let minutes: i32 = parts[1].parse().map_err(|_| "Invalid minute format")?;

    let total_offset = sign * (hours * 3600 + minutes * 60);
    let offset = FixedOffset::east_opt(total_offset).chain_err(|| "Invalid offset");
    let offset_value = offset.unwrap();
    Ok(offset_value)
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{env, fs};

    #[test]
    fn clear_file_test() -> Result<()> {
        let mut dir = env::temp_dir();

        dir.push("foo.txt");
        fs::write(&dir, "some random text")?;

        let s = dir.to_str().ok_or(Error::from(""))?;
        clear_file(&s)?;

        let content = fs::read_to_string(s)?;
        assert_eq!(content, "");
        Ok(())
    }

    #[test]
    fn parse_offset_test() -> Result<()> {
        let offset_str = "+05:30";
        let offset = parse_offset(offset_str)?;
        assert_eq!(offset, FixedOffset::east(19800));
        let offset_str = "-05:30";
        let offset = parse_offset(offset_str)?;
        assert_eq!(offset, FixedOffset::west(19800));
        Ok(())
    }
}
