use crate::services::forecast::types::{
    ForecastAnalysisMeta, ForecastRequest, ForecastResult, ModelDownloadProgress,
};
use crate::services::forecast::{
    catalog, client_chronos, client_nixtla, model_manager, registry, sidecar, storage,
    validation,
};
use serde_json::Value;
use tauri::ipc::Channel;
use tauri::State;

#[tauri::command]
pub async fn run_forecast(
    mut request: ForecastRequest,
    chronos: State<'_, sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    crate::services::forecast::file_input::ensure_request_data(&mut request, None)
        .await
        .map_err(|_| "Impossible de lire les données source".to_string())?;
    validation::validate_request(&request)?;
    let model_id = validation::model_id(&request)?;
    let is_nixtla = model_id.starts_with("timegpt");

    let result = if is_nixtla {
        let key = crate::services::api_keys::get_key("nixtla")
            .map_err(|_| "Clé API Nixtla non configurée".to_string())?;
        client_nixtla::predict(&key, &request, None)
            .await
            .map_err(|_| "Erreur du service de prédiction".to_string())?
    } else {
        if !model_manager::is_installed(model_id) {
            return Err("Modèle non installé".into());
        }
        sidecar::start(&chronos, model_id)
            .await
            .map_err(|_| "Impossible de démarrer le service de prédiction".to_string())?;
        let base_url = sidecar::base_url();
        client_chronos::predict(&base_url, &request, None)
            .await
            .map_err(|_| "Erreur du service de prédiction".to_string())?
    };

    storage::save(&result).await?;
    Ok(result)
}

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
    let configured = crate::services::api_keys::list_configured();
    let providers: Vec<Value> = catalog::FORECAST_PROVIDERS
        .iter()
        .map(|p| {
            let mut v = serde_json::to_value(p).unwrap_or_default();
            if let Some(obj) = v.as_object_mut() {
                obj.insert(
                    "configured".into(),
                    Value::Bool(configured.iter().any(|id| id == p.id)),
                );
            }
            v
        })
        .collect();
    let models: Vec<Value> = catalog::FORECAST_MODELS
        .iter()
        .map(|m| {
            let mut v = serde_json::to_value(m).unwrap_or_default();
            if let Some(obj) = v.as_object_mut() {
                if let Some(runtime) = registry::find_runtime(m.id) {
                    obj.insert(
                        "family_id".into(),
                        Value::String(runtime.family_id.to_string()),
                    );
                    obj.insert(
                        "engine_kind".into(),
                        serde_json::to_value(runtime.engine_kind).unwrap_or_default(),
                    );
                    obj.insert(
                        "capabilities".into(),
                        serde_json::to_value(runtime.capabilities).unwrap_or_default(),
                    );
                }
                obj.insert(
                    "installed".into(),
                    Value::Bool(installed.contains(&m.id.to_string())),
                );
                obj.insert(
                    "size_on_disk".into(),
                    Value::Number(model_manager::get_model_size(m.id).into()),
                );
                obj.insert(
                    "provider_configured".into(),
                    Value::Bool(configured.iter().any(|id| id == m.provider_id)),
                );
            }
            v
        })
        .collect();
    serde_json::json!({
        "providers": providers,
        "models": models,
        "configured_provider_ids": configured
    })
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
