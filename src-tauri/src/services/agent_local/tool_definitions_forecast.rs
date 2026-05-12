use serde_json::Value;

pub fn forecast_tool_definitions() -> Vec<Value> {
    vec![
        super::tool_definitions::tool_def(
            "forecast",
            "Run a time series forecast from structured data. Use this when the user wants to predict future values of a series such as demand, sales, traffic, price, load, or trend. \
             Provide either a JSON array in 'data' or a CSV/Excel path in 'file_path'. \
             The tool returns a compact saved-analysis summary with analysis_id first. \
             Call forecast_read with that analysis_id for predictions and quantiles. \
             Use series_column for multi-series models. Chronos-2 and TimeGPT can use covariates as past columns and optional future-known rows.",
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
                    "series_column": {
                        "type": "string",
                        "description": "Optional series identifier column for multi-series forecasts."
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
             Use action 'annotate' with params.text and params.date to add a note. \
             Use action 'scenario' to add a what-if scenario. \
             For simple scenarios, use params.scenario_kind='percent_adjustment' and params.adjustment_percent. \
             For contextual scenarios, use params.scenario_kind='context_adjustment' and params.covariate_adjustments. \
             Use action 'scenario_update' with params.scenario_id to edit one scenario. \
             Use action 'scenario_delete' with params.scenario_id to delete one scenario. \
             Do not use this tool for decomposition, anomalies, or feature importance yet.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "analysis_id": {
                        "type": "string",
                        "description": "ID of the saved forecast analysis to modify."
                    },
                    "action": {
                        "type": "string",
                        "description": "Action name. Use 'annotate', 'scenario', 'scenario_update', or 'scenario_delete'."
                    },
                    "params": {
                        "type": "object",
                        "description": "Action parameters. annotate requires text and date. scenario requires name plus either adjustment_percent or covariate_adjustments. scenario_update also requires scenario_id. scenario_delete requires scenario_id.",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Annotation text to add to the analysis."
                            },
                            "date": {
                                "type": "string",
                                "description": "Date or timestamp associated with the annotation, ideally ISO format."
                            },
                            "name": {
                                "type": "string",
                                "description": "Scenario name when action is 'scenario' or 'scenario_update'."
                            },
                            "description": {
                                "type": "string",
                                "description": "Optional scenario description for scenario creation or update."
                            },
                            "adjustment_percent": {
                                "type": "number",
                                "description": "Percent adjustment applied to the saved forecast, for example 15 for +15% or -10 for -10%."
                            },
                            "scenario_kind": {
                                "type": "string",
                                "description": "Use percent_adjustment for a derived curve or context_adjustment to modify future-known covariates and rerun the model."
                            },
                            "covariate_adjustments": {
                                "type": "array",
                                "description": "For context_adjustment. Each item modifies one future-known covariate before rerunning the model.",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "column": {"type": "string"},
                                        "mode": {
                                            "type": "string",
                                            "description": "percent or absolute."
                                        },
                                        "value": {"type": "number"}
                                    }
                                }
                            },
                            "target_series_id": {
                                "type": "string",
                                "description": "Optional series id. If omitted, the contextual scenario applies to all series."
                            },
                            "scenario_id": {
                                "type": "string",
                                "description": "Existing scenario id for scenario_update or scenario_delete."
                            }
                        }
                    }
                },
                "required": ["analysis_id", "action", "params"]
            }),
        ),
        super::tool_definitions::tool_def(
            "forecast_read",
            "Read saved forecast analyses. Omit analysis_id, or pass an empty string, to list available analyses. Provide a non-empty analysis_id to read predictions, quantiles, metadata, annotations, and scenarios for one analysis.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "analysis_id": {
                        "type": "string",
                        "description": "Optional. Omit or pass an empty string to list analyses. Provide a non-empty saved analysis id to read one analysis."
                    }
                },
                "required": []
            }),
        ),
    ]
}
