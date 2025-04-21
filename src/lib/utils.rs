use crate::{errors::*, logger::LogTarget};
use chrono::FixedOffset;
use log::{error, info};
use std::{fs, process::Command};

pub fn clear_file(path: &str) -> Result<()> {
    fs::write(path, "")?;
    Ok(())
}

pub fn add_user_to_groups(user: &str, groups: &[String]) -> Result<()> {
    for group in groups {
        if group != user {
            let output = Command::new("usermod")
                .arg("-aG")
                .arg(group)
                .arg(user)
                .output()
                .chain_err(|| {
                    format!(
                        "Failed to execute usermod for user {} and group {}",
                        user, group
                    )
                })?;

            if output.status.success() {
                info!(target: LogTarget::UPDATE.as_str(), "User {} successfully added to group {}", user, group);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(target: LogTarget::UPDATE.as_str(), "usermod failed for user {} and group {}: {}", user, group, stderr.trim());
                return Err(Error::from(format!(
                    "usermod failed for user {} and group {}: {}",
                    user,
                    group,
                    stderr.trim()
                )));
            }
        }
    }
    Ok(())
}

pub fn create_linux_user(username: &str) -> Result<()> {
    let check = Command::new("id").arg(username).status();

    if let Ok(status) = check {
        if status.success() {
            info!(target: LogTarget::UPDATE.as_str(), "User {} already exists, skipping creation.", username);
            return Ok(());
        }
    }

    let status = Command::new("useradd")
        .arg("-m")
        .arg("-d")
        .arg(format!("/home/{}", username))
        .arg("-s")
        .arg("/bin/bash")
        .arg(username)
        .status()
        .chain_err(|| format!("Failed to run useradd command for {}", username))?;

    if status.success() {
        info!(target: LogTarget::UPDATE.as_str(), "User {} added successfully.", username);
        Ok(())
    } else {
        let code = status.code().unwrap_or(-1);
        error!(target: LogTarget::UPDATE.as_str(), "useradd failed for user {} with exit code {}", username, code);
        Err(Error::from(format!(
            "useradd failed for user {} with exit code {}",
            username, code
        )))
    }
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
