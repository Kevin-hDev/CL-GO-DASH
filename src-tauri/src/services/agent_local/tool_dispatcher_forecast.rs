use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::{client_chronos, client_nixtla, storage, model_manager};
use crate::services::forecast::types::ForecastRequest;
use serde_json::Value;
use std::path::Path;

pub async fn dispatch_forecast(
    tool_name: &str,
    args: &Value,
    _working_dir: &Path,
    session_id: &str,
) -> Option<ToolResult> {
    match tool_name {
        "forecast" => Some(handle_forecast(args, session_id).await),
        "forecast_analyze" => Some(handle_analyze(args, session_id).await),
        "forecast_read" => Some(handle_read(args).await),
        _ => None,
    }
}

async fn handle_forecast(args: &Value, session_id: &str) -> ToolResult {
    let request: ForecastRequest = match serde_json::from_value(args.clone()) {
        Ok(r) => r,
        Err(e) => return ToolResult::err(format!("Paramètres invalides: {e}")),
    };

    if request.data.is_none() && request.file_path.is_none() {
        return ToolResult::err(
            "Il faut fournir soit 'data' (JSON) soit 'file_path' (chemin CSV/Excel)"
        );
    }

    let model_id = request.model.as_deref().unwrap_or("chronos-bolt-small");
    let is_nixtla = model_id.starts_with("timegpt");

    let result = if is_nixtla {
        let key = match crate::services::api_keys::get_key("nixtla") {
            Ok(k) => k,
            Err(_) => return ToolResult::err("Clé API Nixtla non configurée"),
        };
        client_nixtla::predict(&key, &request, Some(session_id)).await
    } else {
        if !model_manager::is_installed(model_id) {
            return ToolResult::err(format!(
                "Modèle '{model_id}' non installé. Installez-le depuis les paramètres."
            ));
        }
        let base_url = crate::services::forecast::sidecar::base_url();
        client_chronos::predict(&base_url, &request, Some(session_id)).await
    };

    match result {
        Ok(forecast) => {
            if let Err(e) = storage::save(&forecast).await {
                eprintln!("[forecast] Sauvegarde échouée: {e}");
            }
            match serde_json::to_string_pretty(&forecast) {
                Ok(json) => ToolResult::ok(json),
                Err(e) => ToolResult::err(format!("Sérialisation résultat: {e}")),
            }
        }
        Err(e) => ToolResult::err(e),
    }
}

async fn handle_analyze(args: &Value, _session_id: &str) -> ToolResult {
    let analysis_id = match args["analysis_id"].as_str() {
        Some(id) => id,
        None => return ToolResult::err("Paramètre analysis_id requis"),
    };
    let action = match args["action"].as_str() {
        Some(a) => a,
        None => return ToolResult::err("Paramètre action requis"),
    };

    let analysis = match storage::load(analysis_id).await {
        Ok(a) => a,
        Err(e) => return ToolResult::err(format!("Analyse introuvable: {e}")),
    };

    match action {
        "annotate" => {
            let text = args["params"]["text"].as_str().unwrap_or("");
            let date = args["params"]["date"].as_str().unwrap_or("");
            if text.is_empty() || date.is_empty() {
                return ToolResult::err("params.text et params.date requis");
            }
            let mut updated = analysis;
            updated.annotations.push(
                crate::services::forecast::types::Annotation {
                    id: uuid::Uuid::new_v4().to_string(),
                    date: date.to_string(),
                    text: text.to_string(),
                    source: crate::services::forecast::types::AnnotationSource::Llm,
                },
            );
            if let Err(e) = storage::save(&updated).await {
                return ToolResult::err(format!("Sauvegarde annotation: {e}"));
            }
            ToolResult::ok("Annotation ajoutée")
        }
        _ => ToolResult::err(format!(
            "Action '{action}' pas encore implémentée. Actions disponibles: annotate"
        )),
    }
}

async fn handle_read(args: &Value) -> ToolResult {
    match args["analysis_id"].as_str() {
        Some(id) => match storage::load(id).await {
            Ok(a) => match serde_json::to_string_pretty(&a) {
                Ok(json) => ToolResult::ok(json),
                Err(e) => ToolResult::err(format!("Sérialisation: {e}")),
            },
            Err(e) => ToolResult::err(e),
        },
        None => match storage::list().await {
            Ok(list) => match serde_json::to_string_pretty(&list) {
                Ok(json) => ToolResult::ok(json),
                Err(e) => ToolResult::err(format!("Sérialisation: {e}")),
            },
            Err(e) => ToolResult::err(e),
        },
    }
}
