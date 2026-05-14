use crate::services::forecast::types::{
    ForecastAnalysisMeta, ForecastRequest, ForecastResult, ModelDownloadProgress,
};
use crate::services::forecast::{
    catalog, client_chronos, client_nixtla, model_manager, notes, registry, scenarios, sidecar,
    storage, validation,
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
pub async fn create_forecast_scenario(
    request: scenarios::ScenarioRequest,
    chronos: State<'_, sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    scenarios::create(request, Some(chronos.inner())).await
}

#[tauri::command]
pub async fn update_forecast_scenario(
    request: scenarios::ScenarioUpdateRequest,
    chronos: State<'_, sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    scenarios::update(request, Some(chronos.inner())).await
}

#[tauri::command]
pub async fn delete_forecast_scenario(
    analysis_id: String,
    scenario_id: String,
) -> Result<ForecastResult, String> {
    scenarios::delete(&analysis_id, &scenario_id).await
}

#[tauri::command]
pub async fn delete_forecast_analysis(id: String) -> Result<(), String> {
    storage::delete(&id).await
}

#[tauri::command]
pub async fn rename_forecast_analysis(
    id: String,
    name: String,
) -> Result<ForecastAnalysisMeta, String> {
    storage::rename(&id, &name).await
}

#[tauri::command]
pub async fn list_forecast_notes(analysis_id: String) -> Result<Vec<notes::ForecastNote>, String> {
    notes::list(&analysis_id).await
}

#[tauri::command]
pub async fn create_forecast_note(
    request: notes::ForecastNoteCreateRequest,
) -> Result<notes::ForecastNote, String> {
    notes::create(request).await
}

#[tauri::command]
pub async fn update_forecast_note(
    request: notes::ForecastNoteUpdateRequest,
) -> Result<notes::ForecastNote, String> {
    notes::update(request).await
}

#[tauri::command]
pub async fn delete_forecast_note(analysis_id: String, note_id: String) -> Result<(), String> {
    notes::delete(&analysis_id, &note_id).await
}

#[tauri::command]
pub fn open_forecast_note(analysis_id: String, note_id: String) -> Result<(), String> {
    notes::open(&analysis_id, &note_id)
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
                let runtime = registry::find_runtime(m.id);
                if let Some(runtime) = runtime {
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
                } else {
                    obj.insert(
                        "capabilities".into(),
                        serde_json::json!({
                            "past_covariates": m.covariates,
                            "future_covariates": m.covariates,
                            "multivariate": m.multivariate,
                            "probabilistic": true,
                            "backtesting_ready": false,
                            "anomalies_ready": false,
                            "fine_tuning_ready": false,
                        }),
                    );
                }
                obj.insert(
                    "family_id".into(),
                    Value::String(m.family_id.to_string()),
                );
                obj.insert(
                    "installed".into(),
                    Value::Bool(installed.contains(&m.id.to_string())),
                );
                obj.insert(
                    "installable".into(),
                    Value::Bool(!m.is_cloud && (m.hf_repo.is_some() || m.github_repo.is_some())),
                );
                obj.insert(
                    "runnable".into(),
                    Value::Bool(runtime.is_some()),
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
