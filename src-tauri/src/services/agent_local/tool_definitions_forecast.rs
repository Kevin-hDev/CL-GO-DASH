use serde_json::Value;

#[path = "tool_definitions_forecast_run.rs"]
mod forecast_run;
#[path = "tool_definitions_forecast_audit.rs"]
mod forecast_audit;
#[path = "tool_definitions_forecast_data.rs"]
mod forecast_data;
#[path = "tool_definitions_forecast_evaluation.rs"]
mod forecast_evaluation;

pub fn forecast_tool_definitions() -> Vec<Value> {
    vec![
        forecast_run_definition(),
        forecast_audit::definition(),
        forecast_models_definition(),
        forecast_evaluation::backtest(),
        forecast_evaluation::compare(),
        super::tool_definitions::tool_def(
            "forecast_analyze",
            "Operate on an existing saved forecast analysis. \
             Use action 'annotate' with params.text and params.date to add a note. \
             Use action 'scenario' to add a what-if scenario. \
             For simple scenarios, params.scenario_kind='percent_adjustment' and params.adjustment_percent are required. \
             For contextual scenarios, params.scenario_kind='context_adjustment' and params.covariate_adjustments are required. \
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
                        "description": "Action parameters. annotate requires text and date. scenario requires name, scenario_kind, and the matching scenario parameters. scenario_update also requires scenario_id. scenario_delete requires scenario_id.",
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
            "Read saved forecast analyses. Omit analysis_id to list a bounded set. Provide analysis_id to read one bounded predictions page. Use offset and limit for later pages.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "analysis_id": {
                        "type": "string",
                        "description": "Optional. Omit or pass an empty string to list analyses. Provide a non-empty saved analysis id to read one analysis."
                    },
                    "offset": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "Prediction offset. Defaults to 0."
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 200,
                        "description": "Predictions per page. Defaults to 100 and is capped at 200."
                    }
                },
                "required": []
            }),
        ),
    ]
}

pub(super) fn forecast_run_definition() -> Value {
    forecast_run::definition()
}

fn forecast_models_definition() -> Value {
    let auto = crate::services::forecast::selection_policy::get()
        .unwrap_or_default()
        .mode
        == crate::services::forecast::selection_policy::ForecastSelectionMode::Auto;
    forecast_models_definition_for(auto)
}

fn forecast_models_definition_for(auto: bool) -> Value {
    let properties = if auto {
        serde_json::json!({
            "data_profile_id": {
                "type": "string",
                "maxLength": 64,
                "description": "Validated profile id returned by forecast_data_audit for the current task."
            }
        })
    } else {
        serde_json::json!({})
    };
    super::tool_definitions::tool_def(
        "forecast_models",
        "Inspect the Forecast selection policy. In Manual, use only forced_model. In Auto, pass the validated data_profile_id and choose only one compatible candidate. Prefer rolling backtests when selection_basis is rolling_backtest. Hardware data is exposed only in this Forecast response. Pass the returned selection_id to forecast and never call a capability-only choice the best model.",
        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": if auto { vec!["data_profile_id"] } else { Vec::<&str>::new() }
        }),
    )
}

pub(super) fn definition_for_tool(name: &str) -> Option<Value> {
    forecast_tool_definitions().into_iter().find(|definition| {
        definition
            .pointer("/function/name")
            .and_then(Value::as_str)
            == Some(name)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_models_schema_requires_a_bounded_profile_id() {
        let auto = forecast_models_definition_for(true);
        let manual = forecast_models_definition_for(false);
        let auto_parameters = &auto["function"]["parameters"];

        assert_eq!(auto_parameters["properties"]["data_profile_id"]["maxLength"], 64);
        assert!(auto_parameters["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "data_profile_id"));
        assert!(manual["function"]["parameters"]["required"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}
