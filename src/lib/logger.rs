use std::fs::OpenOptions;
use std::io::Write;
use std::io::Result;
use chrono::{DateTime, Utc};
use crate::config::read_config;
use crate::utils::parse_offset;

pub fn log(filepath: &str, status: &str, message: &str) -> Result<()> {
    let config = match read_config(){
        Ok(config) => config,
        Err(_) => {
            return Ok(());
        },
    };
    if config.logging.debug == "false" {
        return Ok(());
    }
    let offset = parse_offset(&config.logging.offset).expect("Invalid time offset format");
    let now_utc: DateTime<Utc> = Utc::now();
    let local_time = now_utc.with_timezone(&offset);

    let readable_time = local_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let log_message = format!("{} - {} - {}\n", readable_time, status, message);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filepath)?;

    file.write_all(log_message.as_bytes())?;
    Ok(())
}

pub fn logln(message: &str) {
    let config = match read_config(){
        Ok(config) => config,
        Err(_) => {
            log("/opt/watchdog/custom-logs/watchdog.logs", "FAILURE", "Failed to read config").expect("Failed to log");
            return;
        },
    };
    if &config.logging.debug=="false" {
        log("/opt/watchdog/custom-logs/watchdog.logs", "FAILURE", "debug false in logln").expect("Failed to log");
        return;
    }
    let offset = parse_offset(&config.logging.offset).expect("Invalid time offset format");
    let now_utc: DateTime<Utc> = Utc::now();
    let local_time = now_utc.with_timezone(&offset);

    let readable_time = local_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let log_message = format!("{} - {}\n", readable_time, message);

    let filepath = "/opt/watchdog/custom-logs/watchdog.logs";
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filepath).expect("Failed to open log file");

        file.write_all(log_message.as_bytes()).expect("Failed to write to log file");
}

