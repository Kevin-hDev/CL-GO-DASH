use serde_json::Value;

pub fn forecast_tool_definitions() -> Vec<Value> {
    vec![
        super::tool_definitions::tool_def(
            "forecast",
            "Run a time series forecast. Provide data as JSON array or file path (CSV/Excel). \
             Returns predictions with confidence intervals. \
             Use this when the user asks to predict, forecast, or project future values of a time series.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "data": {
                        "type": "string",
                        "description": "JSON array of objects with date and value columns"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Path to CSV or Excel file with time series data"
                    },
                    "target_column": {
                        "type": "string",
                        "description": "Name of the column to predict"
                    },
                    "date_column": {
                        "type": "string",
                        "description": "Name of the date/timestamp column"
                    },
                    "covariate_columns": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Optional covariate column names"
                    },
                    "horizon": {
                        "type": "integer",
                        "description": "Number of future steps to predict"
                    },
                    "frequency": {
                        "type": "string",
                        "description": "Time frequency: D (daily), W (weekly), M (monthly), etc."
                    },
                    "model": {
                        "type": "string",
                        "description": "Model to use (e.g. chronos-bolt-small, timegpt-2-standard)"
                    },
                    "confidence_level": {
                        "type": "number",
                        "description": "Confidence level for prediction intervals (default: 0.9)"
                    }
                },
                "required": ["target_column", "date_column", "horizon", "frequency"]
            }),
        ),
        super::tool_definitions::tool_def(
            "forecast_analyze",
            "Analyze an existing forecast: what-if scenarios, decomposition, anomaly detection, \
             feature importance, or add annotations.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "analysis_id": {
                        "type": "string",
                        "description": "ID of the forecast analysis to work with"
                    },
                    "action": {
                        "type": "string",
                        "description": "Action: what_if, decompose, anomalies, feature_importance, annotate"
                    },
                    "params": {
                        "type": "object",
                        "description": "Action-specific parameters"
                    }
                },
                "required": ["analysis_id", "action"]
            }),
        ),
        super::tool_definitions::tool_def(
            "forecast_read",
            "Read the current state of the forecast panel: list all analyses or get details of one.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "analysis_id": {
                        "type": "string",
                        "description": "ID of a specific analysis (omit to list all)"
                    }
                },
                "required": []
            }),
        ),
    ]
}
