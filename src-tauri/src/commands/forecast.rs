use crate::services::forecast::types::{ForecastAnalysisMeta, ForecastRequest, ForecastResult};
use crate::services::forecast::{
    client_chronos, client_nixtla, model_manager, notes, scenarios, sidecar, storage, validation,
};
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
        let endpoint = sidecar::start(&chronos, model_id)
            .await
            .map_err(|_| "Impossible de démarrer le service de prédiction".to_string())?;
        client_chronos::predict(
            &endpoint.base_url,
            endpoint.auth_token.as_str(),
            &request,
            None,
        )
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
