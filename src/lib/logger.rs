use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Result;

pub fn log(filepath: &str, status: &str, message: &str) -> Result<()> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let timestamp = since_the_epoch.as_secs();

    let log_message = format!("{} - {} - {}\n", timestamp, status, message);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filepath)?;

    file.write_all(log_message.as_bytes())?;
    Ok(())
}