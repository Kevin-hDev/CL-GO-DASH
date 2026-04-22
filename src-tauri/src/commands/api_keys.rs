//! Commandes Tauri pour la gestion des clés API.
//!
//! IMPORTANT : aucune commande ne retourne la clé en clair au frontend.
//! set/delete/has/list/test seulement.

use crate::services::api_keys;

#[tauri::command]
pub async fn set_api_key(provider: String, key: String) -> Result<(), String> {
    api_keys::set_key(&provider, &key)?;
    Ok(())
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
