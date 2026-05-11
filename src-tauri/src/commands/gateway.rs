use crate::models::GatewayConfig;
use crate::services::api_keys;
use crate::services::gateway::service::GatewayService;
use crate::services::gateway::types::GatewayHealth;
use tauri::Emitter;
use zeroize::Zeroize;

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
    let _ = app.emit("gateway-status-changed", state.health().await);
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
    state.update_config(config.clone()).await;

    let mut full = crate::services::config::read_config().unwrap_or_default();
    full.gateway = config;
    crate::services::config::write_config(&full)?;

    let _ = app.emit("gateway-status-changed", state.health().await);
    Ok(())
}

#[tauri::command]
pub async fn gateway_set_token(
    channel_id: String,
    account_id: String,
    mut token: String,
) -> Result<(), String> {
    let vault_key = format!("gateway.{channel_id}.{account_id}");
    let result = api_keys::set_raw(&vault_key, &token);
    token.zeroize();
    result
}

#[tauri::command]
pub async fn gateway_delete_token(channel_id: String, account_id: String) -> Result<(), String> {
    let vault_key = format!("gateway.{channel_id}.{account_id}");
    api_keys::delete_raw(&vault_key)
}

#[tauri::command]
pub async fn gateway_has_token(channel_id: String, account_id: String) -> Result<bool, String> {
    let vault_key = format!("raw:gateway.{channel_id}.{account_id}");
    Ok(api_keys::has_key(&vault_key))
}
