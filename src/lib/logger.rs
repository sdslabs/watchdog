use fern::{Dispatch, InitError};
use log::{LevelFilter, Log};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{collections::HashMap, fs, io, sync::Mutex};

use crate::config::read_config;
use crate::constants::LOG_PATH;
use crate::utils::parse_offset;

pub enum LogTarget {
    UPDATE,
    AUTH,
    SUDO,
    WATCHDOG,
    SSH,
    SU,
    Other(String),
}

impl LogTarget {
    pub fn as_str(&self) -> &str {
        match self {
            LogTarget::UPDATE => "update",
            LogTarget::AUTH => "auth",
            LogTarget::SSH => "ssh",
            LogTarget::SUDO => "sudo",
            LogTarget::SU => "su",
            LogTarget::WATCHDOG => "watchdog",
            LogTarget::Other(_) => "watchdog",
        }
    }
}

fn verbosity_to_level_filter(verbosity: &str) -> LevelFilter {
    match verbosity {
        "v" => LevelFilter::Info,
        "vv" => LevelFilter::Debug,
        "vvv" => LevelFilter::Trace,
        _ => LevelFilter::Warn,
    }
}

lazy_static::lazy_static! {
    static ref TARGET_LOGGERS: Mutex<HashMap<String, Box<dyn Log>>> = Mutex::new(HashMap::new());
}

pub fn init_logger() -> Result<(), InitError> {
    let config =
        read_config().map_err(|_| InitError::from(io::Error::other("Could not read config")))?;

    if config.logging.debug == "false" {
        return Ok(());
    }
    let global_level = verbosity_to_level_filter(&config.logging.verbosity);

    let offset = parse_offset(&config.logging.offset)
        .map_err(|_| InitError::from(io::Error::other("Invalid offset in config")))?;

    let base_dir = LOG_PATH;
    fs::create_dir_all(base_dir).map_err(|e| {
        InitError::from(io::Error::other(format!(
            "Failed to create log directory: {e}"
        )))
    })?;

    let logger = Box::new(PerTargetLogger {
        base_dir: base_dir.to_string(),
        offset,
        global_level,
    });

    log::set_boxed_logger(logger).map(|()| log::set_max_level(global_level))?;

    Ok(())
}

fn classify_target(raw_target: &str) -> String {
    match raw_target {
        "update" | "auth" | "ssh" | "sudo" | "su" | "watchdog" => raw_target.to_string(),
        _ => "dependencies".to_string(), // Third-party logs go here
    }
}

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
struct PerTargetLogger {
    base_dir: String,
    offset: FixedOffset,
    global_level: LevelFilter,
}

impl log::Log for PerTargetLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.global_level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let raw_target = if record.target().is_empty() {
            "watchdog"
        } else {
            record.target()
        };

        let target = classify_target(raw_target);

        let mut loggers = TARGET_LOGGERS.lock().unwrap();

        if !loggers.contains_key(&target) {
            let log_path = format!("{}/{}.logs", self.base_dir, target);
            match fern::log_file(&log_path) {
                Ok(file) => {
                    let (_level_filter, logger): (_, Box<dyn Log>) = Dispatch::new()
                        .level(LevelFilter::Info)
                        .format({
                            let offset = self.offset;
                            move |out, message, record| {
                                let time = chrono::Utc::now()
                                    .with_timezone(&offset)
                                    .format("%Y-%m-%d %H:%M:%S");
                                out.finish(format_args!(
                                    "{} [{}] [{}] {}",
                                    time,
                                    record.level(),
                                    record.target(),
                                    message
                                ))
                            }
                        })
                        .chain(file)
                        .into_log();

                    loggers.insert(target.clone(), logger);
                }
                Err(e) => {
                    eprintln!("Failed to create log file for {target}: {e}");
                    return;
                }
            }
        }

        if let Some(logger) = loggers.get(&target) {
            logger.log(record);
        }
    }

    fn flush(&self) {}
}

pub fn handle_logs_for(component: &str, level: Option<&str>) {
    let path = format!("{LOG_PATH}/{component}.logs");
    let path = Path::new(&path);

    if !path.exists() {
        eprintln!("Log file for component '{component}' does not exist.");
        return;
    }

    let file = File::open(path).expect("Unable to open log file");
    let reader = BufReader::new(file);

    let filter_level = level.map(|lvl| lvl.to_uppercase());

    for line in reader.lines() {
        let line = line.unwrap_or_default();

        if let Some(start) = line.find('[') {
            if let Some(end) = line.find(']') {
                let level_in_line = &line[start + 1..end];

                if let Some(ref lvl) = filter_level {
                    if level_in_line == lvl {
                        println!("{line}");
                    }
                } else {
                    println!("{line}");
                }
            }
        }
    }
}

pub fn handle_logs_all(level: Option<&str>) {
    let log_dir = Path::new(LOG_PATH);
    let mut all_logs = Vec::new();
    println!("Fetching logs from directory: {}", log_dir.display());

    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            println!("Processing file: {}", entry.path().display());
            if let Ok(file) = fs::File::open(entry.path()) {
                let reader = io::BufReader::new(file);

                for line in reader.lines().flatten() {
                    if let Some((timestamp_str, rest)) = line.split_once(' ') {
                        let full_ts = timestamp_str.to_string()
                            + " "
                            + rest.split_whitespace().next().unwrap_or("");

                        if let Ok(naive_dt) =
                            NaiveDateTime::parse_from_str(&full_ts, "%Y-%m-%d %H:%M:%S")
                        {
                            let datetime: DateTime<Utc> = Utc.from_utc_datetime(&naive_dt);

                            let message = line[full_ts.len()..].trim_start();

                            let passes_level_filter = match level {
                                Some(target_level) => {
                                    let formatted_level =
                                        format!("[{}]", target_level.to_uppercase());
                                    message.contains(&formatted_level)
                                }
                                None => true,
                            };

                            if passes_level_filter {
                                all_logs.push((datetime, line.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Sorting logs by timestamp...{}", all_logs.len());
    all_logs.sort_by_key(|(dt, _)| *dt);

    for (_, log_line) in all_logs {
        println!("{log_line}");
    }
}
