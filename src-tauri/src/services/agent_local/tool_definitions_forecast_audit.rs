use crate::services::agent_local::tool_definitions;
use serde_json::Value;

pub(super) fn definition() -> Value {
    let mut properties = super::forecast_data::properties();
    properties.remove("data_profile_id");
    tool_definitions::tool_def(
        "forecast_data_audit",
        "Audit Forecast data before prediction. It validates dates, chronological order, duplicates, missing periods, frequency, history length, series count, future rows, numeric values, outliers, and the prediction budget. Pass data or file_path. A valid audit returns a reusable data_profile_id; use that id in forecast instead of sending the raw data again.",
        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": ["target_column", "date_column", "horizon", "frequency"]
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_requires_mapping_but_never_accepts_an_existing_profile() {
        let definition = definition();
        let parameters = &definition["function"]["parameters"];
        assert!(parameters["properties"].get("data_profile_id").is_none());
        for required in ["target_column", "date_column", "horizon", "frequency"] {
            assert!(parameters["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == required));
        }
    }
}
