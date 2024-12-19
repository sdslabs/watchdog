use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Result;

pub fn log(filetype: &str, status: &str, message: &str) -> Result<()> {
    let filename = match filetype {
        "ssh" => "/opt/watchdog/custom-logs/ssh.logs",
        "sudo" => "/opt/watchdog/custom-logs/sudo.logs",
        "su" => "/opt/watchdog/custom-logs/su.logs",
        "auth" => "/opt/watchdog/custom-logs/auth.logs",
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filetype")),
    };

    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let timestamp = since_the_epoch.as_secs();

    let log_message = format!("{} - {} - {}\n", timestamp, status, message);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)?;

    file.write_all(log_message.as_bytes())?;
    Ok(())
}