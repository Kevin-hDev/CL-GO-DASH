use crate::services::forecast::{catalog, model_manager, storage};
use crate::services::forecast::types::{
    ForecastAnalysisMeta, ForecastResult, ModelDownloadProgress,
};
use serde_json::Value;
use tauri::ipc::Channel;

#[tauri::command]
pub async fn list_forecast_analyses() -> Result<Vec<ForecastAnalysisMeta>, String> {
    storage::list().await
}

#[tauri::command]
pub async fn get_forecast_analysis(id: String) -> Result<ForecastResult, String> {
    storage::load(&id).await
}

#[tauri::command]
pub async fn delete_forecast_analysis(id: String) -> Result<(), String> {
    storage::delete(&id).await
}

#[tauri::command]
pub fn list_forecast_models() -> Value {
    let installed = model_manager::installed_models();
    let providers: Vec<Value> = catalog::FORECAST_PROVIDERS
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();
    let models: Vec<Value> = catalog::FORECAST_MODELS
        .iter()
        .map(|m| {
            let mut v = serde_json::to_value(m).unwrap_or_default();
            if let Some(obj) = v.as_object_mut() {
                obj.insert(
                    "installed".into(),
                    Value::Bool(installed.contains(&m.id.to_string())),
                );
                obj.insert(
                    "size_on_disk".into(),
                    Value::Number(model_manager::get_model_size(m.id).into()),
                );
            }
            v
        })
        .collect();
    serde_json::json!({ "providers": providers, "models": models })
}

#[tauri::command]
pub async fn install_forecast_model(
    name: String,
    on_progress: Channel<ModelDownloadProgress>,
) -> Result<(), String> {
    model_manager::install(&name, &on_progress).await
}

#[tauri::command]
pub async fn uninstall_forecast_model(name: String) -> Result<(), String> {
    model_manager::uninstall(&name).await
}

#[tauri::command]
pub fn list_forecast_providers_catalog() -> Vec<Value> {
    catalog::FORECAST_PROVIDERS
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect()
}
