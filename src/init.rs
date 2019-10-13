extern crate simplelog;
use crate::config;
use simplelog::*;
use std::fs::OpenOptions;

pub fn init(config: &config::Config) {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        OpenOptions::new()
            .read(true)
            .append(true)
            .open(&config.error_log_file)
            .unwrap(),
    )])
    .unwrap();
}
