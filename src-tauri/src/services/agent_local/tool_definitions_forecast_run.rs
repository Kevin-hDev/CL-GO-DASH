use crate::services::agent_local::tool_definitions;
use crate::services::forecast::selection_policy::{self, ForecastSelectionMode};
use serde_json::{Map, Value};

pub(super) fn definition() -> Value {
    let policy = selection_policy::get().unwrap_or_default();
    definition_for(policy)
}

fn definition_for(policy: selection_policy::ForecastSelectionPolicy) -> Value {
    let (policy_text, auto) = match policy.mode {
        ForecastSelectionMode::Manual => {
            let text = match policy.manual_model_id {
                Some(model) => format!(
                    "The Forecast selector forces '{model}'. You must not pass another model id or modify this policy."
                ),
                None => "No manual Forecast model is selected. You must ask the user to select one before you run forecast.".to_string(),
            };
            (text, false)
        }
        ForecastSelectionMode::Auto => (
            "Auto is active. You must call forecast_models before the first forecast or after the task changes, choose only one returned candidate, and pass its id in model. You must not call it the best model without comparable backtests or modify the user's policy.".to_string(),
            true,
        ),
    };
    let mut properties = base_properties();
    if auto {
        properties.insert(
            "model".into(),
            serde_json::json!({
                "type": "string",
                "description": "Model id selected from the bounded forecast_models candidates list."
            }),
        );
    }
    let mut required = vec!["target_column", "date_column", "horizon", "frequency"];
    if auto {
        required.push("model");
    }
    let description = format!(
        "Run a time series forecast from structured data. You provide either a JSON array in data or a CSV/Excel path in file_path. The tool returns a saved analysis_id; you call forecast_read with it for predictions and quantiles. {policy_text} You use series_column for multi-series data and covariate_columns only when the selected model supports them."
    );
    tool_definitions::tool_def(
        "forecast",
        &description,
        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required
        }),
    )
}

#[cfg(test)]
#[path = "tool_definitions_forecast_run_tests.rs"]
mod tests;

fn base_properties() -> Map<String, Value> {
    serde_json::from_value(serde_json::json!({
        "data": {
            "type": "string",
            "description": "JSON row array. Historical rows include date and target; future-known rows may omit target."
        },
        "file_path": {
            "type": "string",
            "description": "CSV or Excel path used instead of data."
        },
        "target_column": {"type": "string", "description": "Target column name."},
        "date_column": {"type": "string", "description": "Date or timestamp column name."},
        "series_column": {"type": "string", "description": "Optional series identifier column."},
        "covariate_columns": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Optional compatible past or future-known context columns."
        },
        "horizon": {"type": "integer", "description": "Future steps to predict."},
        "frequency": {"type": "string", "description": "Frequency such as D, W, M, Q, Y, H, or T."},
        "confidence_level": {"type": "number", "description": "Prediction interval confidence, usually 0.9."}
    }))
    .unwrap_or_default()
}
