use lib::errors::*;
use watchdog_utils::services::github_service;

pub async fn handle_update() -> Result<()> {
    github_service::process_update_request()
        .await
        .map_err(|e| e.to_string().into())
}
