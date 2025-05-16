use crate::{constants::HOME_DIR, errors::*, logger::LogTarget};
use chrono::FixedOffset;
use log::{error, info};
use regex::Regex;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    process::Command,
};
pub fn clear_file(path: &str) -> Result<()> {
    fs::write(path, "")?;
    Ok(())
}

pub fn add_user_to_groups(user: &str, groups: &[String]) -> Result<()> {
    for group in groups {
        let mut target_group = group.as_str();
        if group == "sudo" {
            if !group_exists("sudo") && group_exists("wheel") {
                target_group = "wheel";
            }
        }

        if target_group != user {
            let output = Command::new("usermod")
                .arg("-aG")
                .arg(target_group)
                .arg(user)
                .output()
                .chain_err(|| {
                    format!(
                        "Failed to execute usermod for user {} and group {}",
                        user, target_group
                    )
                })?;

            if output.status.success() {
                info!(target: LogTarget::UPDATE.as_str(), "User {} successfully added to group {}", user, target_group);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(target: LogTarget::UPDATE.as_str(), "usermod failed for user {} and group {}: {}", user, target_group, stderr.trim());
                return Err(Error::from(format!(
                    "usermod failed for user {} and group {}: {}",
                    user,
                    target_group,
                    stderr.trim()
                )));
            }
        }
    }
    Ok(())
}

pub fn remove_user_from_groups(user: &str, groups: &[String]) -> Result<()> {
    for group in groups {
        let mut target_group = group.as_str();
        if group == "sudo" {
            if !group_exists("sudo") && group_exists("wheel") {
                target_group = "wheel";
            }
        }

        if target_group != user {
            let output = Command::new("gpasswd")
                .arg("-d")
                .arg(user)
                .arg(target_group)
                .output()
                .chain_err(|| {
                    format!(
                        "Failed to execute gpasswd for user {} and group {}",
                        user, target_group
                    )
                })?;

            if output.status.success() {
                info!(
                    target: LogTarget::UPDATE.as_str(),
                    "User {} successfully removed from group {}",
                    user, target_group
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(
                    target: LogTarget::UPDATE.as_str(),
                    "gpasswd failed for user {} and group {}: {}",
                    user,
                    target_group,
                    stderr.trim()
                );
                return Err(Error::from(format!(
                    "gpasswd failed for user {} and group {}: {}",
                    user,
                    target_group,
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
        .arg(format!("{}/{}", HOME_DIR, username))
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

pub fn delete_user(user: &str) -> Result<()> {
    let output = Command::new("sudo")
        .arg("userdel")
        .arg("-r")
        .arg(user)
        .output()?;

    if output.status.success() {
        info!("User '{}' deleted successfully.", user);
        Ok(())
    } else {
        error!(
            "Failed to delete user '{}': {}",
            user,
            String::from_utf8_lossy(&output.stderr)
        );
        Err(Error::from(format!(
            "Failed to delete user '{}': {}",
            user,
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub fn update_user_bashrc(user: &str) -> Result<()> {
    let bashrc_path = format!("{}/{}/.bashrc", HOME_DIR, user);
    let bashrc_lines = r#"
# Load group-specific config if present
for group in $(id -nG "$USER"); do
    group_bashrc="/home/$group/.bashrc"
    [ -f "$group_bashrc" ] && source "$group_bashrc"
done
cd /home
"#;

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&bashrc_path)?;

    file.write_all(bashrc_lines.as_bytes())?;
    info!("Appended group-config loader to '{}'.", bashrc_path);

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

fn group_exists(group: &str) -> bool {
    fs::read_to_string("/etc/group")
        .map(|content| {
            content
                .lines()
                .any(|line| line.starts_with(&format!("{}:", group)))
        })
        .unwrap_or(false)
}

pub fn user_exists(username: &str) -> bool {
    match Command::new("id").arg(username).status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

pub fn extract_sudo_command() -> Result<String> {
    let pid = std::process::id();
    let status_path = format!("/proc/{}/status", pid);

    let parent_pid = fs::read_to_string(&status_path)?
        .lines()
        .find(|line| line.starts_with("PPid:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or("Could not find PPid in /proc/[pid]/status")?
        .parse::<u32>()
        .chain_err(|| "Failed to parse PPid")?;

    let cmdline_path = format!("/proc/{}/cmdline", parent_pid);
    let cmdline = fs::read(&cmdline_path)
        .map(|bytes| {
            bytes
                .split(|b| *b == 0)
                .map(|part| String::from_utf8_lossy(part).to_string())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_else(|_| "UNKNOWN".to_string());

    Ok(cmdline)
}

pub fn extract_diff_parts(diff_data: &str) -> Vec<(String, String, String, String)> {
    let re_access = Regex::new(r"diff --git a/(access/([^/]+)/([^/]+)/([\w\d]+))").unwrap();
    let re_names = Regex::new(r"diff --git a/(names/([\w\d]+))").unwrap();
    let mut parts_with_status = HashMap::new();
    for line in diff_data.lines() {
        if let Some(caps) = re_access.captures(line) {
            let full_path = &caps[1];
            let project = &caps[2];
            let provider = &caps[3];
            let hash = &caps[4];
            let status = if diff_data.contains("new file mode") && line.contains(full_path) {
                "added"
            } else if diff_data.contains("deleted file mode") && line.contains(full_path) {
                "deleted"
            } else {
                "modified"
            };
            info!(
                "Access file change detected: {}/{}/{}, status: {}",
                project, provider, hash, status
            );
            parts_with_status
                .entry((project.to_string(), provider.to_string(), hash.to_string()))
                .or_insert(status.to_string());
        } else if let Some(caps) = re_names.captures(line) {
            let full_path = &caps[1];
            let hash = &caps[2];
            let status = if diff_data.contains("deleted file mode") && line.contains(full_path) {
                "deleteduser"
            } else {
                "modifieduser"
            };
            info!("Name file change detected: {}, status: {}", hash, status);
            parts_with_status
                .entry(("".to_string(), "names".to_string(), hash.to_string()))
                .or_insert(status.to_string());
        }
    }
    parts_with_status
        .into_iter()
        .map(|((proj, prov, hash), status)| (proj, prov, hash, status))
        .collect()
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
