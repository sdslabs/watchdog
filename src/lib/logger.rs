use fern::{Dispatch, InitError};
use log::{LevelFilter, Log};
use std::{
    collections::HashMap,
    fs, io,
    sync::Mutex,
};

use crate::config::read_config;
use crate::utils::parse_offset;

lazy_static::lazy_static! {
    static ref TARGET_LOGGERS: Mutex<HashMap<String, Box<dyn Log>>> = Mutex::new(HashMap::new());
}

pub fn init_logger() -> Result<(), InitError> {
    let config = read_config().map_err(|_| {
        InitError::from(io::Error::new(
            io::ErrorKind::Other,
            "Could not read config",
        ))
    })?;

    if config.logging.debug == "false" {
        return Ok(()); // No logging
    }

    let offset = parse_offset(&config.logging.offset).map_err(|_| {
        InitError::from(io::Error::new(
            io::ErrorKind::Other,
            "Invalid offset in config",
        ))
    })?;

    let base_dir = "/opt/watchdog/custom-logs";
    fs::create_dir_all(base_dir).map_err(|e| {
        InitError::from(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create log directory: {}", e),
        ))
    })?;

    let logger = Box::new(PerTargetLogger {
        base_dir: base_dir.to_string(),
        offset,
    });

    log::set_boxed_logger(logger)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(InitError::SetLoggerError)
}

use chrono::FixedOffset;

struct PerTargetLogger {
    base_dir: String,
    offset: FixedOffset,
}

impl log::Log for PerTargetLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= LevelFilter::Info
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let target = if record.target().is_empty(){
            "watchdog"
        }else{
            record.target()
        };
        let mut loggers = TARGET_LOGGERS.lock().unwrap();

        if !loggers.contains_key(target) {
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
                                    "{} [{}] {}",
                                    time,
                                    record.level(),
                                    message
                                ))
                            }
                        })
                        .chain(file)
                        .into_log();

                    loggers.insert(target.to_string(), logger);
                }
                Err(e) => {
                    eprintln!("Failed to create log file for {}: {}", target, e);
                    return;
                }
            }
        }

        if let Some(logger) = loggers.get(target) {
            logger.log(record);
        }
    }

    fn flush(&self) {}
}
