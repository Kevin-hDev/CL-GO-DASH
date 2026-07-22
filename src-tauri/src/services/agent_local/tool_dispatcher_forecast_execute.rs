use crate::services::forecast::types::{ForecastRequest, ForecastResult};
use crate::services::forecast::{
    client_chronos, client_nixtla, model_manager, registry, sidecar,
};
use tauri::{Emitter, Manager};

pub async fn run_cloud(
    request: &ForecastRequest,
    session_id: &str,
) -> Result<ForecastResult, String> {
    let key = crate::services::api_keys::get_key("nixtla")
        .map_err(|_| "Clé API Nixtla non configurée".to_string())?;
    client_nixtla::predict(&key, request, Some(session_id)).await
}

pub async fn run_local(
    request: &ForecastRequest,
    model_id: &str,
    runtime: &registry::ForecastRuntimeSpec,
    session_id: &str,
) -> Result<ForecastResult, String> {
    if !model_manager::is_installed(model_id) {
        return Err("Modèle non installé".into());
    }
    let spec = crate::services::forecast::catalog::find_model(model_id).ok_or("Modèle inconnu")?;
    crate::services::forecast::hardware_profile::validate_model_resources(spec)?;
    let app = super::app_handle_global::get().ok_or("Service de prédiction indisponible")?;
    let chronos = app.state::<sidecar::ChronosSidecar>();
    let endpoint = sidecar::start(chronos.inner(), model_id, runtime.family_id)
        .await
        .map_err(|_| "Impossible de démarrer le service de prédiction".to_string())?;
    let result = client_chronos::predict(
        &endpoint.base_url,
        endpoint.auth_token.as_str(),
        request,
        Some(session_id),
    )
    .await;
    sidecar::schedule_idle_stop(chronos.inner());
    result
}

pub fn emit_created(analysis_id: &str, session_id: &str) {
    if let Some(app) = super::app_handle_global::get() {
        let _ = app.emit(
            "forecast-analysis-created",
            serde_json::json!({ "analysis_id": analysis_id, "session_id": session_id }),
        );
    }
}
