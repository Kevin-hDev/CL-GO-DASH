use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::types::ForecastRequest;
use crate::services::forecast::{
    client_chronos, client_nixtla, model_manager, registry, selected_model, sidecar, storage,
    validation,
};
use serde_json::Value;
use std::path::Path;
use tauri::{Emitter, Manager};

pub async fn dispatch_forecast(
    tool_name: &str,
    args: &Value,
    _working_dir: &Path,
    session_id: &str,
) -> Option<ToolResult> {
    match tool_name {
        "forecast" => Some(handle_forecast(args, _working_dir, session_id).await),
        "forecast_models" => Some(super::tool_dispatcher_forecast_models::handle()),
        "forecast_analyze" => Some(super::tool_dispatcher_forecast_analyze::handle(args).await),
        "forecast_read" => Some(handle_read(args).await),
        _ => None,
    }
}

async fn handle_forecast(args: &Value, working_dir: &Path, session_id: &str) -> ToolResult {
    let mut request: ForecastRequest = match serde_json::from_value(args.clone()) {
        Ok(r) => r,
        Err(e) => return ToolResult::err(format!("Paramètres invalides: {e}")),
    };
    let requested_model = request.model.clone();
    crate::services::forecast::request_normalize::normalize_request(&mut request);
    let selected_model = match selected_model::apply_required(&mut request) {
        Ok(model) => model,
        Err(e) => return ToolResult::err(e),
    };

    if request.data.is_none() && request.file_path.is_none() {
        return forced_model_error(
            &selected_model,
            requested_model.as_deref(),
            "Il faut fournir soit 'data' (JSON) soit 'file_path' (chemin CSV/Excel)",
        );
    }

    if let Err(e) =
        crate::services::forecast::file_input::ensure_request_data(&mut request, Some(working_dir))
            .await
    {
        return forced_model_error(&selected_model, requested_model.as_deref(), &e);
    }

    if let Err(e) = validation::validate_request(&request) {
        return forced_model_error(&selected_model, requested_model.as_deref(), &e);
    }
    let model_id = match validation::model_id(&request) {
        Ok(id) => id,
        Err(e) => return forced_model_error(&selected_model, requested_model.as_deref(), &e),
    };
    let runtime = match registry::find_runtime(model_id) {
        Some(runtime) => runtime,
        None => {
            return forced_model_error(
                &selected_model,
                requested_model.as_deref(),
                "Moteur indisponible",
            )
        }
    };
    if !registry::has_predict_adapter(runtime) {
        return forced_model_error(
            &selected_model,
            requested_model.as_deref(),
            "Moteur indisponible",
        );
    }

    let result = if registry::is_cloud(runtime) {
        let key = match crate::services::api_keys::get_key("nixtla") {
            Ok(k) => k,
            Err(_) => {
                return forced_model_error(
                    &selected_model,
                    requested_model.as_deref(),
                    "Clé API Nixtla non configurée",
                )
            }
        };
        client_nixtla::predict(&key, &request, Some(session_id)).await
    } else {
        if !model_manager::is_installed(model_id) {
            return forced_model_error(
                &selected_model,
                requested_model.as_deref(),
                "Modèle non installé",
            );
        }
        let Some(app) = super::app_handle_global::get() else {
            return forced_model_error(
                &selected_model,
                requested_model.as_deref(),
                "Service de prédiction indisponible",
            );
        };
        let chronos = app.state::<sidecar::ChronosSidecar>();
        let endpoint = match sidecar::start(chronos.inner(), model_id, runtime.family_id).await {
            Ok(endpoint) => endpoint,
            Err(_) => {
                return forced_model_error(
                    &selected_model,
                    requested_model.as_deref(),
                    "Impossible de démarrer le service de prédiction",
                )
            }
        };
        client_chronos::predict(
            &endpoint.base_url,
            endpoint.auth_token.as_str(),
            &request,
            Some(session_id),
        )
        .await
    };

    match result {
        Ok(forecast) => {
            if storage::save(&forecast).await.is_err() {
                return forced_model_error(
                    &selected_model,
                    requested_model.as_deref(),
                    "Sauvegarde de la prévision échouée",
                );
            }
            if let Some(app) = super::app_handle_global::get() {
                let _ = app.emit(
                    "forecast-analysis-created",
                    serde_json::json!({
                        "analysis_id": forecast.id,
                        "session_id": session_id,
                    }),
                );
            }
            match super::tool_dispatcher_forecast_output::created_payload(&forecast) {
                Ok(json) => ToolResult::ok(json),
                Err(e) => ToolResult::err(e),
            }
        }
        Err(e) => forced_model_error(&selected_model, requested_model.as_deref(), &e),
    }
}

fn forced_model_error(
    selected_model: &str,
    requested_model: Option<&str>,
    error: &str,
) -> ToolResult {
    let ignored = requested_model.filter(|model| *model != selected_model);
    let payload = serde_json::json!({
        "error": error,
        "model_selection": {
            "mode": "selector_forced",
            "effective_model": selected_model,
            "requested_model_ignored": ignored,
            "selector_locked": true,
            "next_step": "Corriger la requête ou demander à l'utilisateur de changer le modèle dans le sélecteur Forecast."
        }
    });
    match serde_json::to_string_pretty(&payload) {
        Ok(json) => ToolResult::err(json),
        Err(_) => ToolResult::err(error),
    }
}

async fn handle_read(args: &Value) -> ToolResult {
    match args["analysis_id"].as_str() {
        Some(id) if !id.trim().is_empty() => match storage::load(id.trim()).await {
            Ok(a) => match super::tool_dispatcher_forecast_output::analysis_payload(&a) {
                Ok(json) => ToolResult::ok(json),
                Err(e) => ToolResult::err(e),
            },
            Err(e) => ToolResult::err(e),
        },
        _ => match storage::list().await {
            Ok(list) => match super::tool_dispatcher_forecast_output::list_payload(&list) {
                Ok(json) => ToolResult::ok(json),
                Err(e) => ToolResult::err(e),
            },
            Err(e) => ToolResult::err(e),
        },
    }
}
