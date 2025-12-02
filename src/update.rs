use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::logger::LogTarget;
use watchdog_utils_II::services::github_service;

pub fn handle_update() -> Result<()> {
    let config = read_config()?;
    init(&config)?;

    let keyhouse_config = config.keyhouse;
    let hostname = config.hostname;

    let rt = tokio::runtime::Runtime::new()?;
    if let Err(e) = rt.block_on(github_service::process_update_request(
        keyhouse_config,
        LogTarget::UPDATE.as_str(),
        hostname,
    )) {
        return Err(e.to_string().into());
    }
    Ok(())
}
