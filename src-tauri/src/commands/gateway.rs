use crate::models::GatewayConfig;
use crate::services::gateway::service::GatewayService;
use crate::services::gateway::tokens;
use crate::services::gateway::types::{ChannelStatus, GatewayHealth};
use tauri::Emitter;

#[tauri::command]
pub async fn gateway_status(
    state: tauri::State<'_, GatewayService>,
) -> Result<GatewayHealth, String> {
    Ok(state.health().await)
}

#[tauri::command]
pub async fn gateway_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, GatewayService>,
) -> Result<(), String> {
    let config = state.config().await;
    state.start(config, app).await;
    Ok(())
}

#[tauri::command]
pub async fn gateway_stop(
    app: tauri::AppHandle,
    state: tauri::State<'_, GatewayService>,
) -> Result<(), String> {
    state.stop().await;
    let health = state.health().await;
    for channel in &health.channels {
        let _ = app.emit("gateway-channel-status", channel);
    }
    let _ = app.emit("gateway-status-changed", health);
    Ok(())
}

#[tauri::command]
pub async fn gateway_get_config(
    state: tauri::State<'_, GatewayService>,
) -> Result<GatewayConfig, String> {
    let disk = crate::services::config::read_config()
        .unwrap_or_default()
        .gateway;
    state.update_config(disk.clone()).await;
    Ok(disk)
}

#[tauri::command]
pub async fn gateway_set_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, GatewayService>,
    config: GatewayConfig,
) -> Result<(), String> {
    let current_health = state.health().await;
    let should_restart = current_health.running
        && current_health
            .channels
            .iter()
            .any(|c| matches!(c.status, ChannelStatus::Starting | ChannelStatus::Running));
    state.update_config(config.clone()).await;

    let mut full = crate::services::config::read_config().unwrap_or_default();
    full.gateway = config.clone();
    crate::services::config::write_config(&full)?;

    if should_restart && config.enabled {
        state.start(config, app).await;
        return Ok(());
    }
    if should_restart {
        state.stop().await;
    }
    let _ = app.emit("gateway-status-changed", state.health().await);
    Ok(())
}

#[tauri::command]
pub async fn gateway_set_token(
    channel_id: String,
    account_id: String,
    token_kind: String,
    token: String,
) -> Result<(), String> {
    tokens::set(&channel_id, &account_id, &token_kind, token)
}

#[tauri::command]
pub async fn gateway_delete_token(
    channel_id: String,
    account_id: String,
    token_kind: Option<String>,
) -> Result<(), String> {
    tokens::delete(&channel_id, &account_id, token_kind.as_deref())
}

#[tauri::command]
pub async fn gateway_has_token(
    channel_id: String,
    account_id: String,
    token_kind: String,
) -> Result<bool, String> {
    tokens::has(&channel_id, &account_id, &token_kind)
}
