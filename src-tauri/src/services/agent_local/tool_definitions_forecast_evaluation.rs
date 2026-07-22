use crate::services::forecast::limits::{MAX_BACKTEST_MODELS, MAX_BACKTEST_WINDOWS};
use serde_json::Value;

pub(super) fn backtest() -> Value {
    super::super::tool_definitions::tool_def(
        "forecast_backtest",
        "Run bounded rolling temporal validation on one saved analysis. It evaluates Naive, Seasonal Naive, Drift, ETS and the requested forecast models on identical windows. Use the saved analysis_id returned by forecast. Results are persisted and compact; never call a model best unless this tool reports that it beats the baselines.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "analysis_id": {
                    "type": "string",
                    "maxLength": 64,
                    "description": "Saved Forecast analysis id."
                },
                "model_ids": {
                    "type": "array",
                    "maxItems": MAX_BACKTEST_MODELS,
                    "items": { "type": "string", "maxLength": 80 },
                    "description": "Optional model ids to evaluate. Defaults to the analysis model."
                },
                "max_windows": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": MAX_BACKTEST_WINDOWS,
                    "description": "Maximum rolling windows. Defaults to 3."
                }
            },
            "required": ["analysis_id"]
        }),
    )
}

pub(super) fn compare() -> Value {
    super::super::tool_definitions::tool_def(
        "forecast_compare_models",
        "Read the comparable rolling-backtest ranking saved on one Forecast analysis. It returns bounded metrics, baseline status and measured interval coverage without returning raw folds or data.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "analysis_id": {
                    "type": "string",
                    "maxLength": 64,
                    "description": "Saved analysis containing backtest results."
                }
            },
            "required": ["analysis_id"]
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backtest_schema_is_bounded_and_comparison_requires_an_analysis() {
        let backtest = backtest();
        let compare = compare();
        assert_eq!(
            backtest["function"]["parameters"]["properties"]["model_ids"]["maxItems"],
            MAX_BACKTEST_MODELS
        );
        assert_eq!(
            backtest["function"]["parameters"]["properties"]["max_windows"]["maximum"],
            MAX_BACKTEST_WINDOWS
        );
        assert_eq!(
            compare["function"]["parameters"]["required"][0],
            "analysis_id"
        );
    }
}
