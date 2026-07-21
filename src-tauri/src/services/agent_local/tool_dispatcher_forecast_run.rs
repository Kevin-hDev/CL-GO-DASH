use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::types::ForecastRequest;
use crate::services::forecast::{
    client_chronos, client_nixtla, data_profiles, model_manager, registry, selected_model, sidecar,
    storage, validation,
};
use serde_json::Value;
use std::path::Path;
use tauri::{Emitter, Manager};

pub async fn handle(args: &Value, working_dir: &Path, session_id: &str) -> ToolResult {
    let mut request: ForecastRequest = match serde_json::from_value(args.clone()) {
        Ok(request) => request,
        Err(_) => return ToolResult::err("Paramètres Forecast invalides"),
    };
    let requested_model = request.model.clone();
    crate::services::forecast::request_normalize::normalize_request(&mut request);
    let selected = match selected_model::apply_required(&mut request) {
        Ok(model) => model,
        Err(error) => return forced_model_error("", requested_model.as_deref(), &error),
    };
    if let Err(error) = data_profiles::hydrate_request(&mut request).await {
        return forced_model_error(&selected, requested_model.as_deref(), &error);
    }
    if request.data.is_none() && request.file_path.is_none() {
        return forced_model_error(
            &selected,
            requested_model.as_deref(),
            "Il faut fournir data, file_path ou data_profile_id",
        );
    }
    if let Err(error) =
        crate::services::forecast::file_input::ensure_request_data(&mut request, Some(working_dir))
            .await
    {
        return forced_model_error(&selected, requested_model.as_deref(), &error);
    }
    if let Err(error) = validation::validate_request(&request) {
        return forced_model_error(&selected, requested_model.as_deref(), &error);
    }
    let data_profile =
        match crate::services::forecast::data_quality::validate_and_bind(&mut request) {
            Ok(profile) => profile,
            Err(error) => return forced_model_error(&selected, requested_model.as_deref(), &error),
        };
    let model_id = match validation::model_id(&request) {
        Ok(id) => id,
        Err(error) => return forced_model_error(&selected, requested_model.as_deref(), &error),
    };
    let runtime = match registry::find_runtime(model_id) {
        Some(runtime) if registry::has_predict_adapter(runtime) => runtime,
        _ => {
            return forced_model_error(
                &selected,
                requested_model.as_deref(),
                "Moteur indisponible",
            )
        }
    };
    if data_profile.future_rows > 0
        && !request.covariate_columns.is_empty()
        && !runtime.capabilities.future_covariates
    {
        return forced_model_error(
            &selected,
            requested_model.as_deref(),
            "Variables futures non supportées par ce moteur",
        );
    }

    let result = if registry::is_cloud(runtime) {
        run_cloud(&request, session_id).await
    } else {
        run_local(&request, model_id, runtime, session_id).await
    };
    let forecast = match result {
        Ok(forecast) => forecast,
        Err(error) => return forced_model_error(&selected, requested_model.as_deref(), &error),
    };
    if let Some(profile) = &forecast.data_profile {
        if data_profiles::save(profile, &request).await.is_err() {
            return forced_model_error(
                &selected,
                requested_model.as_deref(),
                "Sauvegarde du profil de données échouée",
            );
        }
    }
    if storage::save(&forecast).await.is_err() {
        return forced_model_error(
            &selected,
            requested_model.as_deref(),
            "Sauvegarde de la prévision échouée",
        );
    }
    emit_created(&forecast.id, session_id);
    match super::tool_dispatcher_forecast_output::created_payload(&forecast) {
        Ok(json) => ToolResult::ok(json),
        Err(error) => ToolResult::err(error),
    }
}

async fn run_cloud(request: &ForecastRequest, session_id: &str) -> Result<crate::services::forecast::types::ForecastResult, String> {
    let key = crate::services::api_keys::get_key("nixtla")
        .map_err(|_| "Clé API Nixtla non configurée".to_string())?;
    client_nixtla::predict(&key, request, Some(session_id)).await
}

async fn run_local(
    request: &ForecastRequest,
    model_id: &str,
    runtime: &registry::ForecastRuntimeSpec,
    session_id: &str,
) -> Result<crate::services::forecast::types::ForecastResult, String> {
    if !model_manager::is_installed(model_id) {
        return Err("Modèle non installé".into());
    }
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

fn forced_model_error(selected: &str, requested: Option<&str>, error: &str) -> ToolResult {
    let ignored = requested.filter(|model| *model != selected);
    let payload = serde_json::json!({
        "error": error,
        "model_selection": {
            "mode": "selector_forced",
            "effective_model": selected,
            "requested_model_ignored": ignored,
            "selector_locked": true,
            "next_step": "Corriger la requête ou demander à l'utilisateur de changer le modèle dans le sélecteur Forecast."
        }
    });
    serde_json::to_string_pretty(&payload)
        .map_or_else(|_| ToolResult::err(error), ToolResult::err)
}

fn emit_created(analysis_id: &str, session_id: &str) {
    if let Some(app) = super::app_handle_global::get() {
        let _ = app.emit(
            "forecast-analysis-created",
            serde_json::json!({ "analysis_id": analysis_id, "session_id": session_id }),
        );
    }
}
