use lib::config::read_config;
use lib::errors::*;
use lib::init::init;
use lib::logger::LogTarget;
use watchdog_utils_II::config::{Config as UtilsConfig, KeyhouseConf as UtilsKeyhouseConf};
use watchdog_utils_II::services::github_service;

pub fn handle_update(update_all: bool) -> Result<()> {
    let config = read_config()?;
    init(&config)?;
    let utils_keyhouse = UtilsKeyhouseConf {
        base_url: config.keyhouse.base_url.clone(),
        token: config.keyhouse.token.clone(),
    };
    let utils_config = UtilsConfig::new(config.hostname.clone(), utils_keyhouse, config.cache_path);
    let rt = tokio::runtime::Runtime::new()?;
    if let Err(e) = rt.block_on(async {
        if update_all {
            github_service::process_full_update_request(utils_config, LogTarget::UPDATE.as_str())
                .await
        } else {
            github_service::process_update_request(utils_config, LogTarget::UPDATE.as_str(), false)
                .await
        }
    }) {
        return Err(e.to_string().into());
    }
    Ok(())
}
