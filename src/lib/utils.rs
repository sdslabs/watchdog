use std::fs;

use crate::errors::*;

pub const AUTH_LOG_PATH: &str = "/opt/watchdog/custom-logs/auth.logs";
pub const SSH_LOG_PATH: &str = "/opt/watchdog/custom-logs/ssh.logs";
pub const SUDO_LOG_PATH: &str = "/opt/watchdog/custom-logs/sudo.logs";
pub const SU_LOG_PATH: &str = "/opt/watchdog/custom-logs/su.logs";

pub fn clear_file(path: &str) -> Result<()> {
    fs::write(path, "")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

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
}
