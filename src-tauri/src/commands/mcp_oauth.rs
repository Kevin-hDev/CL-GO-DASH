use crate::services::api_keys;
use crate::services::mcp_oauth::{flow, storage};
use zeroize::Zeroize;

fn validate_connector_id(id: &str) -> Result<(), String> {
    crate::services::mcp_bridge::config::validate_connector_id(id)
}

#[tauri::command]
pub async fn start_mcp_oauth(
    app: tauri::AppHandle,
    connector_id: String,
    endpoint: String,
) -> Result<(), String> {
    validate_connector_id(&connector_id)?;
    if endpoint.is_empty() || !endpoint.starts_with("https://") {
        return Err("endpoint MCP non HTTPS".to_string());
    }
    if !crate::services::mcp_bridge::registry::is_trusted_endpoint_pub(&connector_id, &endpoint) {
        return Err("endpoint non autorisé pour OAuth".to_string());
    }
    tauri::async_runtime::spawn(flow::run(app, connector_id, endpoint));
    Ok(())
}

#[tauri::command]
pub async fn cancel_mcp_oauth(connector_id: String) -> Result<(), String> {
    validate_connector_id(&connector_id)?;
    flow::cancel(&connector_id);
    Ok(())
}

#[tauri::command]
pub async fn has_mcp_oauth_token(connector_id: String) -> Result<bool, String> {
    validate_connector_id(&connector_id)?;
    Ok(storage::has_tokens(&connector_id))
}

#[tauri::command]
pub async fn delete_mcp_oauth_token(connector_id: String) -> Result<(), String> {
    validate_connector_id(&connector_id)?;
    crate::services::mcp_bridge::registry::invalidate_cache(&connector_id);
    storage::delete_tokens(&connector_id)
}

fn validate_env_key(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 64 {
        return Err("identifiant invalide".to_string());
    }
    if !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err("identifiant invalide".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn set_mcp_env_token(
    connector_id: String,
    env_key: String,
    mut value: String,
) -> Result<(), String> {
    validate_connector_id(&connector_id)?;
    validate_env_key(&env_key)?;
    let vault_key = format!("mcp_{connector_id}_{}", env_key.to_lowercase());
    let result = api_keys::set_key_raw(&vault_key, &value);
    value.zeroize();
    if result.is_ok() {
        crate::services::mcp_bridge::registry::invalidate_cache(&connector_id);
    }
    result
}

#[tauri::command]
pub async fn delete_mcp_env_token(connector_id: String, env_key: String) -> Result<(), String> {
    validate_connector_id(&connector_id)?;
    validate_env_key(&env_key)?;
    crate::services::mcp_bridge::registry::invalidate_cache(&connector_id);
    let vault_key = format!("mcp_{connector_id}_{}", env_key.to_lowercase());
    api_keys::delete_key_raw(&vault_key)
}
