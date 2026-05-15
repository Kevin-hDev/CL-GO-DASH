use crate::services::forecast::types::ModelDownloadProgress;
use crate::services::forecast::{catalog, model_listing, model_manager, validation};
use serde_json::Value;
use tauri::ipc::Channel;

#[tauri::command]
pub fn list_forecast_models() -> Value {
    model_listing::list_models()
}

#[tauri::command]
pub async fn install_forecast_model(
    name: String,
    on_progress: Channel<ModelDownloadProgress>,
) -> Result<(), String> {
    validation::validate_model_id(&name)?;
    model_manager::install(&name, &on_progress).await
}

#[tauri::command]
pub async fn uninstall_forecast_model(name: String) -> Result<(), String> {
    validation::validate_model_id(&name)?;
    model_manager::uninstall(&name).await
}

#[tauri::command]
pub fn list_forecast_providers_catalog() -> Vec<Value> {
    catalog::FORECAST_PROVIDERS
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect()
}
