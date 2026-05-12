use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::scenarios::{ScenarioRequest, ScenarioUpdateRequest};
use crate::services::forecast::{scenarios, sidecar, storage};
use serde_json::Value;
use tauri::Manager;

pub async fn handle(args: &Value) -> ToolResult {
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
        "annotate" => annotate(analysis, args).await,
        "scenario" => scenario_create(analysis_id, &args["params"]).await,
        "scenario_update" => scenario_update(analysis_id, &args["params"]).await,
        "scenario_delete" => scenario_delete(analysis_id, &args["params"]).await,
        _ => ToolResult::err(format!(
            "Action '{action}' pas encore implémentée. Actions disponibles: annotate, scenario, scenario_update, scenario_delete"
        )),
    }
}

async fn annotate(
    mut analysis: crate::services::forecast::types::ForecastResult,
    args: &Value,
) -> ToolResult {
    let text = args["params"]["text"].as_str().unwrap_or("");
    let date = args["params"]["date"].as_str().unwrap_or("");
    if text.is_empty() || date.is_empty() {
        return ToolResult::err(
            "Paramètres d'annotation manquants. Utiliser params.text et params.date.",
        );
    }
    analysis
        .annotations
        .push(crate::services::forecast::types::Annotation {
            id: uuid::Uuid::new_v4().to_string(),
            date: date.to_string(),
            text: text.to_string(),
            source: crate::services::forecast::types::AnnotationSource::Llm,
        });
    match storage::save(&analysis).await {
        Ok(_) => ToolResult::ok("Annotation ajoutée"),
        Err(e) => ToolResult::err(format!("Sauvegarde annotation: {e}")),
    }
}

async fn scenario_create(analysis_id: &str, params: &Value) -> ToolResult {
    let name = params["name"].as_str().unwrap_or("");
    let request = ScenarioRequest {
        analysis_id: analysis_id.to_string(),
        name: name.to_string(),
        description: params["description"].as_str().map(str::to_string),
        scenario_kind: scenario_kind(params),
        adjustment_percent: params["adjustment_percent"].as_f64().unwrap_or(0.0),
        covariate_adjustments: serde_json::from_value(params["covariate_adjustments"].clone())
            .unwrap_or_default(),
        target_series_id: params["target_series_id"].as_str().map(str::to_string),
    };
    let chronos = forecast_chronos();
    save_scenario_result(scenarios::create(request, chronos.as_deref()).await)
}

async fn scenario_update(analysis_id: &str, params: &Value) -> ToolResult {
    let scenario_id = params["scenario_id"].as_str().unwrap_or("");
    let name = params["name"].as_str().unwrap_or("");
    let request = ScenarioUpdateRequest {
        analysis_id: analysis_id.to_string(),
        scenario_id: scenario_id.to_string(),
        name: name.to_string(),
        description: params["description"].as_str().map(str::to_string),
        scenario_kind: scenario_kind(params),
        adjustment_percent: params["adjustment_percent"].as_f64().unwrap_or(0.0),
        covariate_adjustments: serde_json::from_value(params["covariate_adjustments"].clone())
            .unwrap_or_default(),
        target_series_id: params["target_series_id"].as_str().map(str::to_string),
    };
    let chronos = forecast_chronos();
    save_scenario_result(scenarios::update(request, chronos.as_deref()).await)
}

async fn scenario_delete(analysis_id: &str, params: &Value) -> ToolResult {
    let scenario_id = params["scenario_id"].as_str().unwrap_or("");
    if scenario_id.is_empty() {
        return ToolResult::err("Paramètres de scénario manquants. Utiliser params.scenario_id.");
    }
    save_scenario_result(scenarios::delete(analysis_id, scenario_id).await)
}

fn scenario_kind(params: &Value) -> String {
    params["scenario_kind"]
        .as_str()
        .unwrap_or("percent_adjustment")
        .to_string()
}

fn save_scenario_result(
    result: Result<crate::services::forecast::types::ForecastResult, String>,
) -> ToolResult {
    match result {
        Ok(updated) => match super::tool_dispatcher_forecast_output::analysis_payload(&updated) {
            Ok(json) => ToolResult::ok(json),
            Err(e) => ToolResult::err(e),
        },
        Err(e) => ToolResult::err(e),
    }
}

fn forecast_chronos() -> Option<tauri::State<'static, sidecar::ChronosSidecar>> {
    let app = super::app_handle_global::get()?;
    Some(app.state::<sidecar::ChronosSidecar>())
}
