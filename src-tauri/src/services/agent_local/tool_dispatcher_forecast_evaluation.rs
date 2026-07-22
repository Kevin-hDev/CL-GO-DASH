use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::evaluation::BacktestRequest;
use crate::services::forecast::sidecar::ChronosSidecar;
use crate::services::forecast::storage;
use serde_json::Value;
use tauri::{Emitter, Manager};

pub async fn backtest(args: &Value) -> ToolResult {
    let request: BacktestRequest = match serde_json::from_value(args.clone()) {
        Ok(request) => request,
        Err(_) => return ToolResult::err("Paramètres de backtest invalides"),
    };
    let Some(app) = super::app_handle_global::get() else {
        return ToolResult::err("Service de backtest indisponible");
    };
    let chronos = app.state::<ChronosSidecar>();
    match crate::services::forecast::evaluation::run(request, chronos.inner()).await {
        Ok(analysis) => {
            let _ = app.emit(
                "forecast-analysis-updated",
                serde_json::json!({ "analysis_id": analysis.id, "session_id": analysis.session_id }),
            );
            comparison_payload(&analysis)
        }
        Err(error) => ToolResult::err(error),
    }
}

pub async fn compare(args: &Value) -> ToolResult {
    let Some(id) = args["analysis_id"].as_str().filter(|id| !id.trim().is_empty()) else {
        return ToolResult::err("Identifiant d'analyse requis");
    };
    match storage::load(id.trim()).await {
        Ok(analysis) => comparison_payload(&analysis),
        Err(error) => ToolResult::err(error),
    }
}

fn comparison_payload(analysis: &crate::services::forecast::types::ForecastResult) -> ToolResult {
    let Some(evaluation) = &analysis.evaluation else {
        return ToolResult::err("Aucun backtest comparable pour cette analyse");
    };
    let mut results: Vec<_> = evaluation.results.iter().collect();
    results.sort_by_key(|result| result.rank.unwrap_or(usize::MAX));
    let payload = serde_json::json!({
        "analysis_id": analysis.id,
        "ranking_method": "rolling_backtest",
        "horizon": evaluation.horizon,
        "windows": evaluation.windows,
        "warning": evaluation.warning,
        "results": results.into_iter().map(|result| serde_json::json!({
            "model_id": result.model_id,
            "kind": result.kind,
            "rank": result.rank,
            "metrics": result.metrics,
            "calibration": result.calibration,
            "beats_best_baseline": result.beats_best_baseline,
            "duration_ms": result.duration_ms,
            "warning": result.warning,
        })).collect::<Vec<_>>(),
        "usage": "Prefer the lowest MASE among successful results and require an advanced model to beat the best baseline."
    });
    match serde_json::to_string(&payload) {
        Ok(json) => ToolResult::ok(json),
        Err(_) => ToolResult::err("Résultat de comparaison indisponible"),
    }
}
