use serde_json::Value;

pub fn forecast_tool_definitions() -> Vec<Value> {
    vec![
        super::tool_definitions::tool_def(
            "forecast",
            "Run a time series forecast from structured data. Use this when the user wants to predict future values of a series such as demand, sales, traffic, price, load, or trend. \
             Provide either a JSON array in 'data' or a CSV/Excel path in 'file_path'. \
             The tool returns a saved forecast analysis with predictions and quantiles. \
             For Chronos-2, covariates can be included as past columns and optional future-known rows.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "data": {
                        "type": "string",
                        "description": "JSON array of row objects. Historical rows must include date and target. Optional future rows may omit the target and keep known future covariates."
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Path to a CSV or Excel file. Use this instead of 'data' when the source already exists on disk."
                    },
                    "target_column": {
                        "type": "string",
                        "description": "Name of the target column to forecast."
                    },
                    "date_column": {
                        "type": "string",
                        "description": "Name of the date or timestamp column."
                    },
                    "covariate_columns": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Optional context columns. For Chronos-2 they can be used as past covariates and future-known covariates if future rows are provided."
                    },
                    "horizon": {
                        "type": "integer",
                        "description": "Number of future steps to predict."
                    },
                    "frequency": {
                        "type": "string",
                        "description": "Time frequency such as D, W, M, Q, Y, H, or T."
                    },
                    "model": {
                        "type": "string",
                        "description": "Forecast model id, for example chronos-bolt-small, chronos-2, or timegpt-2-standard."
                    },
                    "confidence_level": {
                        "type": "number",
                        "description": "Confidence level for prediction intervals, usually 0.9."
                    }
                },
                "required": ["target_column", "date_column", "horizon", "frequency"]
            }),
        ),
        super::tool_definitions::tool_def(
            "forecast_analyze",
            "Operate on an existing saved forecast analysis. \
             The only supported action right now is 'annotate', which adds a note to a forecast analysis. \
             Do not use this tool for scenarios, decomposition, anomalies, or feature importance yet.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "analysis_id": {
                        "type": "string",
                        "description": "ID of the saved forecast analysis to modify."
                    },
                    "action": {
                        "type": "string",
                        "description": "Action name. Use 'annotate'. Other action names are not implemented yet."
                    },
                    "params": {
                        "type": "object",
                        "description": "Action parameters. For 'annotate', provide 'text' and 'date'."
                    }
                },
                "required": ["analysis_id", "action"]
            }),
        ),
        super::tool_definitions::tool_def(
            "forecast_read",
            "Read saved forecast analyses. Use this to list available forecast analyses or fetch the full content of a specific analysis before explaining, comparing, or annotating it.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "analysis_id": {
                        "type": "string",
                        "description": "Optional analysis id. Omit it to list saved analyses, or provide it to read one analysis in full."
                    }
                },
                "required": []
            }),
        ),
    ]
}
