use crate::services::agent_local::tool_definitions;
use crate::services::forecast::selection_policy::{self, ForecastSelectionMode};
use serde_json::Value;

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
            "Auto is active. After forecast_data_audit, you must call forecast_models with the returned data_profile_id before the first forecast or after the task changes, choose only one returned candidate, and pass its id in model. You must not call it the best model without comparable backtests or modify the user's policy.".to_string(),
            true,
        ),
    };
    let mut properties = super::forecast_data::properties();
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
        "Run a validated time series forecast. For every new dataset, you must call forecast_data_audit first and then pass its reusable data_profile_id instead of resending raw data. Direct data or file_path remains available for application compatibility. The tool returns a saved analysis_id; you call forecast_read with it for paginated predictions and quantiles. {policy_text} You use series_column for multi-series data and covariate_columns only when the selected model supports them."
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
