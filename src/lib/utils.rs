use crate::{config::Config, constants::HOME_DIR, errors::*, logger::LogTarget};
use chrono::FixedOffset;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use log::{debug, error, info};
use regex::Regex;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
};
pub fn clear_file(path: &str) -> Result<()> {
    fs::write(path, "")?;
    Ok(())
}

pub fn add_user_to_groups(user: &str, groups: &[String]) -> Result<()> {
    for group in groups {
        let mut target_group = group.as_str();
        if group == "sudo" && !group_exists("sudo") && group_exists("wheel") {
            target_group = "wheel";
        }

        if target_group != user {
            let output = Command::new("usermod")
                .arg("-aG")
                .arg(target_group)
                .arg(user)
                .output()
                .chain_err(|| {
                    format!("Failed to execute usermod for user {user} and group {target_group}")
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
        if group == "sudo" && !group_exists("sudo") && group_exists("wheel") {
            target_group = "wheel";
        }

        if target_group != user {
            let output = Command::new("gpasswd")
                .arg("-d")
                .arg(user)
                .arg(target_group)
                .output()
                .chain_err(|| {
                    format!("Failed to execute gpasswd for user {user} and group {target_group}")
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
        .arg(format!("{HOME_DIR}/{username}"))
        .arg("-s")
        .arg("/bin/bash")
        .arg(username)
        .status()
        .chain_err(|| format!("Failed to run useradd command for {username}"))?;

    if status.success() {
        info!(target: LogTarget::UPDATE.as_str(), "User {} added successfully.", username);
        Ok(())
    } else {
        let code = status.code().unwrap_or(-1);
        error!(target: LogTarget::UPDATE.as_str(), "useradd failed for user {} with exit code {}", username, code);
        Err(Error::from(format!(
            "useradd failed for user {username} with exit code {code}"
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
    let bashrc_path = format!("{HOME_DIR}/{user}/.bashrc");
    let bashrc_lines = r#"
# Load group-specific config [WATCHDOG]
for group in $(id -nG "$USER"); do
    group_home="/home/$group"
    group_bashrc="/home/$group/.bashrc"
    if [ -f "$group_bashrc" ]; then
        OLD_HOME="$HOME"
        HOME="$group_home"
        source "$group_bashrc"
        HOME="$OLD_HOME"
    fi
done
cd /home
"#;

    if let Ok(contents) = std::fs::read_to_string(&bashrc_path) {
        if contents.contains("Load group-specific config [WATCHDOG]") {
            info!(
                "Group-config loader already present in '{}'. Skipping append.",
                bashrc_path
            );
            return Ok(());
        }
    }

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
                .any(|line| line.starts_with(&format!("{group}:")))
        })
        .unwrap_or(false)
}

pub fn user_exists(username: &str) -> bool {
    match Command::new("id").arg(username).status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Attempts to find the full sudo command executed by a user using various fallback methods.
pub fn extract_sudo_command() -> Result<String> {
    // Method 1: SUDO_COMMAND environment variable (Primary method)
    if let Ok(cmd) = std::env::var("SUDO_COMMAND") {
        debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Checking Method 1 (SUDO_COMMAND env var)");
        if !cmd.is_empty() {
            info!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Succeeded using Method 1 (SUDO_COMMAND env var)");
            debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 1 found command: {}", cmd);
            return Ok(cmd);
        } else {
            debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 1 -> SUDO_COMMAND env var was empty");
        }
    } else {
        debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 1 -> SUDO_COMMAND env var was not set");
    }

    // Method 2: Parse /proc filesystem to find parent sudo process
    let pid = std::process::id();
    debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 1 failed; Current PID: {}; Checking Method 2 (Parent PID cmdline)", pid);

    let status_path = format!("/proc/{pid}/status");
    let status_content =
        fs::read_to_string(&status_path).chain_err(|| format!("Failed to read {status_path}"))?;

    let parent_pid_str = status_content
        .lines()
        .find(|line| line.starts_with("PPid:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or("Could not find PPid in /proc status")?;

    let parent_pid = parent_pid_str
        .parse::<u32>()
        .chain_err(|| "Failed to parse PPid")?;
    debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 2 -> Found Parent PID: {}", parent_pid);

    if let Ok(cmd) = extract_command_from_pid(parent_pid) {
        info!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Succeeded using Method 2 (Parent PID cmdline)");
        debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 2 succeeded with command: {}", cmd);
        return Ok(cmd);
    } else {
        debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 2 -> didn't found command from parent_pid: {parent_pid}");
    }

    // Method 3: Walk up the process tree to find sudo
    debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 2 failed; Checking Method 3 (Process tree walk)");
    let mut current_pid = parent_pid;
    for i in 0..5 {
        debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 3: Checking PID {} (level {})", current_pid, i);
        if let Ok(cmd) = extract_command_from_pid(current_pid) {
            info!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Succeeded using Method 3 (Process tree walk at level {})", i);
            debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 3 succeeded with command: {}", cmd);
            return Ok(cmd);
        }

        // Get parent of current process
        let status_path = format!("/proc/{current_pid}/status");
        if let Ok(status) = fs::read_to_string(&status_path) {
            if let Some(line) = status.lines().find(|l| l.starts_with("PPid:")) {
                if let Some(ppid_str) = line.split_whitespace().nth(1) {
                    if let Ok(ppid) = ppid_str.parse::<u32>() {
                        if ppid <= 1 {
                            debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 3: Reached PID <= 1, stopping tree walk");
                            break;
                        }
                        current_pid = ppid;
                        continue;
                    }
                }
            }
        }
        debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 3: Failed to get PPid for {}, stopping tree walk", current_pid);
        break;
    }

    // Method 4: Fall back to current process cmdline
    debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 3 failed. Checking Method 4 (Current process cmdline)");
    let our_cmdline_path = format!("/proc/{pid}/cmdline");
    let our_cmdline =
        fs::read(&our_cmdline_path).chain_err(|| "Failed to read current process cmdline")?;

    let our_args: Vec<String> = our_cmdline
        .split(|&b| b == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).to_string())
        .collect();

    if !our_args.is_empty() {
        let cmd = our_args.join(" ");
        info!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Succeeded using Method 4 (Current process cmdline)");
        debug!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: Method 4 succeeded with command: {}", cmd);
        return Ok(cmd);
    }

    error!(target: LogTarget::SUDO.as_str(), "extract_sudo_command: All methods failed to determine command");
    bail!("Could not determine sudo command from any method")
}

/// Helper function to extract command from a specific PID
fn extract_command_from_pid(pid: u32) -> Result<String> {
    let cmdline_path = format!("/proc/{pid}/cmdline");
    let cmdline_bytes =
        fs::read(&cmdline_path).chain_err(|| format!("Failed to read {cmdline_path}"))?;

    // Parse cmdline: arguments are separated by null bytes
    let args: Vec<String> = cmdline_bytes
        .split(|&b| b == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).to_string())
        .collect();

    if args.is_empty() {
        bail!("Process cmdline is empty");
    }

    let process_name = args[0].split('/').next_back().unwrap_or(&args[0]);
    if process_name == "sudo" || process_name == "pkexec" {
        debug!(target: LogTarget::SUDO.as_str(), "extract_command_from_pid: PID {} is 'sudo' or 'pkexec'. Returning full command.", pid);
    } else {
        debug!(target: LogTarget::SUDO.as_str(), "extract_command_from_pid: PID {} is not 'sudo'. Returning full command anyway.", pid);
    }

    let cmd = args.join(" ");
    debug!(target: LogTarget::SUDO.as_str(), "extract_command_from_pid: PID {} full command: {}", pid, cmd);
    Ok(cmd)
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
                target: LogTarget::UPDATE.as_str(),"Access file change detected: {}/{}/{}, status: {}",
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
            info!(target: LogTarget::UPDATE.as_str(),"Name file change detected: {}, status: {}", hash, status);
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

pub fn check_local_cache(config: &Config, user: &str, ssh_key: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.input_str(ssh_key);
    let hex = hasher.result_str();

    let cache_path = &config.cache_path;

    // Check Name
    let name_path = Path::new(cache_path).join("names").join(&hex);
    if !name_path.exists() {
        error!(target: LogTarget::AUTH.as_str(), "Cache Miss: Name file not found for hash {}", hex);
        return false;
    }

    let cached_user = match fs::read_to_string(&name_path) {
        Ok(u) => u.trim().to_string(),
        Err(e) => {
            error!(target: LogTarget::AUTH.as_str(), "Failed to read cache file {:?}: {}", name_path, e);
            return false;
        }
    };

    if cached_user != user {
        error!(target: LogTarget::AUTH.as_str(), "Cache Mismatch: User '{}' requested, but key hash belongs to '{}'", user, cached_user);
        return false;
    }

    // Check Access
    let host_access_path = Path::new(cache_path).join("access").join(&config.hostname);
    if !host_access_path.exists() {
        error!(target: LogTarget::AUTH.as_str(), "Cache Miss: No access directory for host '{}'", config.hostname);
        return false;
    }

    match fs::read_dir(host_access_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let hash_path = entry.path().join(&hex);
                    if hash_path.exists() {
                        info!(target: LogTarget::AUTH.as_str(), "Cache Hit: User '{}' has access via group {:?}", user, entry.file_name());
                        return true;
                    }
                }
            }
        }
        Err(e) => {
            error!(target: LogTarget::AUTH.as_str(), "Cache Error: Failed to read access directory: {}", e);
            return false;
        }
    }

    error!(target: LogTarget::AUTH.as_str(), "Cache Miss: User '{}' found in names, but no access file found for host '{}'", user, config.hostname);
    false
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
        clear_file(s)?;

        let content = fs::read_to_string(s)?;
        assert_eq!(content, "");
        Ok(())
    }

    #[test]
    fn parse_offset_test() -> Result<()> {
        let offset_str = "+05:30";
        let offset = parse_offset(offset_str)?;
        assert_eq!(offset, FixedOffset::east_opt(19800).unwrap());
        let offset_str = "-05:30";
        let offset = parse_offset(offset_str)?;
        assert_eq!(offset, FixedOffset::west_opt(19800).unwrap());
        Ok(())
    }
}
