use crate::services::forecast::types::ForecastResult;
use crate::services::forecast::limits::{
    MAX_TOOL_ANALYSES, MAX_TOOL_ANNOTATIONS, MAX_TOOL_PREDICTIONS,
};
use serde_json::{json, Value};

pub fn created_payload(forecast: &ForecastResult) -> Result<String, String> {
    to_pretty(json!({
        "status": "created",
        "analysis_id": forecast.id,
        "name": forecast.name,
        "model": forecast.model,
        "model_selection": {
            "source": forecast.provenance.selection_source,
            "effective_model": forecast.model,
            "basis": forecast.provenance.selection_basis,
            "reason_codes": forecast.provenance.selection_reason_codes,
            "hardware_class": forecast.provenance.hardware_class
        },
        "provider": forecast.provider,
        "target_column": forecast.target_column,
        "series_column": forecast.input_data.series_column,
        "series_count": forecast.data_profile.as_ref().map(|profile| profile.series_count).unwrap_or(1),
        "series_ids": forecast.input_data.series_ids,
        "horizon": forecast.horizon,
        "frequency": forecast.frequency,
        "confidence_level": forecast.confidence_level,
        "data_profile_id": forecast.data_profile.as_ref().map(|profile| profile.id.as_str()),
        "data_quality_issues": forecast.data_profile.as_ref().map(|profile| &profile.issues),
        "schema_version": forecast.schema_version,
        "revision": forecast.revision,
        "input_points": forecast.input_summary.points,
        "predictions_count": forecast.predictions.len(),
        "covariates_used": forecast.covariates_used,
        "preview": {
            "first_prediction": forecast.predictions.first(),
            "last_prediction": forecast.predictions.last()
        },
        "next_step": "Call forecast_backtest with this analysis_id to compare the model with baselines, then use forecast_compare_models for the compact ranking or forecast_read for predictions."
    }))
}

pub fn analysis_payload(
    analysis: &ForecastResult,
    offset: usize,
    requested_limit: usize,
) -> Result<String, String> {
    let limit = requested_limit.clamp(1, MAX_TOOL_PREDICTIONS);
    let start = offset.min(analysis.predictions.len());
    let end = start.saturating_add(limit).min(analysis.predictions.len());
    let scenarios: Vec<Value> = analysis
        .scenarios
        .iter()
        .map(|scenario| json!({
            "id": scenario.id,
            "name": scenario.name,
            "description": scenario.description,
            "predictions_count": scenario.predictions.len(),
            "params_modified": scenario.params_modified
        }))
        .collect();
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
        "data_profile": analysis.data_profile,
        "schema_version": analysis.schema_version,
        "revision": analysis.revision,
        "provenance": analysis.provenance,
        "predictions": &analysis.predictions[start..end],
        "quantiles": {
            "q10": slice_values(&analysis.quantiles.q10, start, end),
            "q50": slice_values(&analysis.quantiles.q50, start, end),
            "q90": slice_values(&analysis.quantiles.q90, start, end)
        },
        "pagination": {
            "offset": start,
            "limit": limit,
            "returned": end.saturating_sub(start),
            "total": analysis.predictions.len(),
            "has_more": end < analysis.predictions.len()
        },
        "covariates_used": analysis.covariates_used,
        "metrics": analysis.metrics,
        "evaluation_available": analysis.evaluation.is_some(),
        "evaluation_summary": analysis.evaluation.as_ref().map(|evaluation| json!({
            "horizon": evaluation.horizon,
            "windows": evaluation.windows,
            "warning": evaluation.warning,
            "results": evaluation.results.iter().map(|result| json!({
                "model_id": result.model_id,
                "kind": result.kind,
                "rank": result.rank,
                "mase": result.metrics.as_ref().map(|metrics| metrics.mase),
                "beats_best_baseline": result.beats_best_baseline,
                "warning": result.warning,
            })).collect::<Vec<_>>()
        })),
        "annotations": analysis.annotations.iter().take(MAX_TOOL_ANNOTATIONS).collect::<Vec<_>>(),
        "annotations_truncated": analysis.annotations.len() > MAX_TOOL_ANNOTATIONS,
        "scenarios": scenarios
    }))
}

pub fn list_payload(
    list: &[crate::services::forecast::types::ForecastAnalysisMeta],
) -> Result<String, String> {
    let analyses: Vec<_> = list.iter().rev().take(MAX_TOOL_ANALYSES).collect();
    to_pretty(json!({
        "count": list.len(),
        "analyses": analyses,
        "truncated": list.len() > MAX_TOOL_ANALYSES,
        "usage": "Call forecast_read with one analysis_id from this list to read an analysis."
    }))
}

fn slice_values(values: &[f64], start: usize, end: usize) -> &[f64] {
    if values.len() < end {
        return &[];
    }
    &values[start..end]
}

fn to_pretty(value: Value) -> Result<String, String> {
    serde_json::to_string_pretty(&value)
        .map_err(|_| "Résultat Forecast indisponible".to_string())
}

#[cfg(test)]
#[path = "tool_dispatcher_forecast_output_tests.rs"]
mod tests;
