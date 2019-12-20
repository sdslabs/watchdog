// extern crate simplelog;

use crate::config::Config;
// use log::error;
// use simplelog::*;
// use std::fs::OpenOptions;
use crate::errors::*;

pub fn init(_config: &Config) -> Result<()> {
    Ok(())
}

// pub fn init_logger(config: &Config) {
//     let log_file = match OpenOptions::new()
//         .create_new(true)
//         .read(true)
//         .append(true)
//         .open(&config.error_log_file)
//     {
//         Ok(f) => f,
//         Err(_) => {
//             error!("Watchdog: Couldn't open log file");
//             panic!("Watchdog: Couldn't open log file");
//         }
//     };

//     let _res = match CombinedLogger::init(vec![WriteLogger::new(
//         LevelFilter::Info,
//         Config::default(),
//         log_file,
//     )]) {
//         Ok(_) => {}
//         Err(_) => error!("Watchdog: Couldnt start logger for some reason"),
//     };
// }
