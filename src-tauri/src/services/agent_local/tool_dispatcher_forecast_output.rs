use crate::services::forecast::types::ForecastResult;
use serde_json::{json, Value};

pub fn created_payload(forecast: &ForecastResult) -> Result<String, String> {
    to_pretty(json!({
        "status": "created",
        "analysis_id": forecast.id,
        "name": forecast.name,
        "model": forecast.model,
        "model_selection": {
            "mode": "selector_forced",
            "effective_model": forecast.model
        },
        "provider": forecast.provider,
        "target_column": forecast.target_column,
        "series_column": forecast.input_data.series_column,
        "series_count": forecast.input_data.series_ids.len(),
        "series_ids": forecast.input_data.series_ids,
        "horizon": forecast.horizon,
        "frequency": forecast.frequency,
        "input_points": forecast.input_summary.points,
        "predictions_count": forecast.predictions.len(),
        "covariates_used": forecast.covariates_used,
        "preview": {
            "first_prediction": forecast.predictions.first(),
            "last_prediction": forecast.predictions.last()
        },
        "next_step": "Use forecast_read with this analysis_id to read the saved analysis."
    }))
}

pub fn analysis_payload(analysis: &ForecastResult) -> Result<String, String> {
    to_pretty(json!({
        "analysis_id": analysis.id,
        "name": analysis.name,
        "created_at": analysis.created_at,
        "session_id": analysis.session_id,
        "model": analysis.model,
        "provider": analysis.provider,
        "target_column": analysis.target_column,
        "horizon": analysis.horizon,
        "frequency": analysis.frequency,
        "input_summary": analysis.input_summary,
        "input_columns": analysis.input_data.columns,
        "history_points": analysis.input_data.history.len(),
        "predictions": analysis.predictions,
        "quantiles": analysis.quantiles,
        "covariates_used": analysis.covariates_used,
        "metrics": analysis.metrics,
        "annotations": analysis.annotations,
        "scenarios": analysis.scenarios
    }))
}

pub fn list_payload(
    list: &[crate::services::forecast::types::ForecastAnalysisMeta],
) -> Result<String, String> {
    to_pretty(json!({
        "count": list.len(),
        "analyses": list,
        "usage": "Call forecast_read with one analysis_id from this list to read an analysis."
    }))
}

fn to_pretty(value: Value) -> Result<String, String> {
    serde_json::to_string_pretty(&value).map_err(|e| format!("Sérialisation: {e}"))
}
