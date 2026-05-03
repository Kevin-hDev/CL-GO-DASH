//! Commandes Tauri pour la gestion des clés API.
//!
//! IMPORTANT : aucune commande ne retourne la clé en clair au frontend.
//! set/delete/has/list/test seulement.

use crate::services::api_keys;
use zeroize::Zeroize;

#[tauri::command]
pub async fn set_api_key(provider: String, mut key: String) -> Result<(), String> {
    let result = api_keys::set_key(&provider, &key);
    key.zeroize();
    result
}

#[tauri::command]
pub async fn delete_api_key(provider: String) -> Result<(), String> {
    api_keys::delete_key(&provider)?;
    Ok(())
}

#[tauri::command]
pub async fn has_api_key(provider: String) -> Result<bool, String> {
    Ok(api_keys::has_key(&provider))
}

#[tauri::command]
pub async fn list_configured_providers() -> Result<Vec<String>, String> {
    Ok(api_keys::list_configured())
}

#[tauri::command]
pub async fn test_api_key(provider: String) -> Result<(), String> {
    api_keys::test_key(&provider).await
}

#[tauri::command]
pub async fn test_api_key_with_value(provider: String, mut key: String) -> Result<(), String> {
    let result = api_keys::test_key_raw(&provider, &key).await;
    key.zeroize();
    result
}
