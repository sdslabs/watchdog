use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Result;
use chrono::{DateTime, Utc};

pub fn log(filepath: &str, status: &str, message: &str) -> Result<()> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let timestamp = since_the_epoch.as_secs();
    let datetime = DateTime::<Utc>::from(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp));
    let readable_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    let log_message = format!("{} - {} - {}\n", readable_time, status, message);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filepath)?;

    file.write_all(log_message.as_bytes())?;
    Ok(())
}

pub fn logln(message: &str) {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let timestamp = since_the_epoch.as_secs();
    let datetime = DateTime::<Utc>::from(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp));
    let readable_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    let log_message = format!("{} - {}\n", readable_time, message);

    let filepath = "/opt/watchdog/custom-logs/watchdog.logs";
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filepath).expect("Failed to open log file");


        file.write_all(log_message.as_bytes()).expect("Failed to write to log file");
    }