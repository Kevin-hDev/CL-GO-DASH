use crate::services::mcp_bridge::{config, process_manager, registry};
use crate::services::{api_keys, mcp_oauth::storage};
use tauri::Emitter;

#[tauri::command]
pub async fn list_mcp_connectors() -> Result<Vec<config::StoredConnector>, String> {
    config::load()
}

#[tauri::command]
pub async fn add_mcp_connector(
    app: tauri::AppHandle,
    connector: config::StoredConnector,
) -> Result<(), String> {
    let connector_id = connector.id.clone();
    config::upsert(connector)?;
    registry::invalidate_cache(&connector_id);
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn remove_mcp_connector(
    app: tauri::AppHandle,
    connector_id: String,
) -> Result<(), String> {
    let connector = config::find(&connector_id)?;
    delete_connector_secrets(&connector_id, connector.as_ref())?;
    config::remove(&connector_id)?;
    registry::invalidate_cache(&connector_id);
    process_manager::shutdown_one(&connector_id);
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn set_mcp_connector_status(
    app: tauri::AppHandle,
    connector_id: String,
    status: String,
) -> Result<(), String> {
    config::set_status(&connector_id, &status)?;
    registry::invalidate_cache(&connector_id);
    if status == "disconnected" {
        process_manager::shutdown_one(&connector_id);
    }
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn set_mcp_connector_chat_enabled(
    app: tauri::AppHandle,
    connector_id: String,
    enabled: bool,
) -> Result<(), String> {
    config::set_chat_enabled(&connector_id, enabled)?;
    registry::invalidate_cache(&connector_id);
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn test_mcp_connector(connector: config::StoredConnector) -> Result<(), String> {
    registry::test_connector(connector).await
}

fn delete_connector_secrets(
    connector_id: &str,
    connector: Option<&config::StoredConnector>,
) -> Result<(), String> {
    storage::delete_tokens(connector_id)?;
    let Some(connector) = connector else {
        return Ok(());
    };
    let env_keys = config::validated_env_keys(connector.env_keys.as_deref())?;
    for env_key in env_keys {
        let vault_key = format!("mcp_{connector_id}_{}", env_key.to_lowercase());
        api_keys::delete_key_raw(&vault_key)?;
    }
    Ok(())
}
