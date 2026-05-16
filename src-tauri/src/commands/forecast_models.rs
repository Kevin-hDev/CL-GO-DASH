use crate::services::forecast::types::ModelDownloadProgress;
use crate::services::forecast::{
    catalog, model_config, model_listing, model_manager, selected_model, validation,
};
use serde_json::{Map, Value};
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn list_forecast_models() -> Value {
    model_listing::list_models()
}

#[tauri::command]
pub fn get_selected_forecast_model() -> Option<String> {
    selected_model::get()
}

#[tauri::command]
pub fn set_selected_forecast_model(app: AppHandle, name: String) -> Result<(), String> {
    selected_model::set(&name)?;
    let _ = app.emit("forecast-selected-model-changed", name);
    Ok(())
}

#[tauri::command]
pub async fn install_forecast_model(
    app: AppHandle,
    name: String,
    on_progress: Channel<ModelDownloadProgress>,
) -> Result<(), String> {
    validation::validate_model_id(&name)?;
    model_manager::install(&name, &on_progress).await?;
    let _ = app.emit("forecast-models-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn uninstall_forecast_model(app: AppHandle, name: String) -> Result<(), String> {
    validation::validate_model_id(&name)?;
    model_manager::uninstall(&name).await?;
    let _ = app.emit("forecast-models-changed", ());
    Ok(())
}

#[tauri::command]
pub fn list_forecast_providers_catalog() -> Vec<Value> {
    catalog::FORECAST_PROVIDERS
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect()
}

#[tauri::command]
pub fn get_forecast_model_config(
    model_id: String,
) -> Result<model_config::ForecastModelConfig, String> {
    model_config::get(&model_id)
}

#[tauri::command]
pub fn set_forecast_model_config(
    app: AppHandle,
    model_id: String,
    values: Map<String, Value>,
) -> Result<model_config::ForecastModelConfig, String> {
    let config = model_config::set(&model_id, values)?;
    let _ = app.emit("forecast-model-config-changed", &model_id);
    Ok(config)
}
