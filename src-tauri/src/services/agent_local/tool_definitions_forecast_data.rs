use serde_json::{Map, Value};

pub(super) fn properties() -> Map<String, Value> {
    serde_json::from_value(serde_json::json!({
        "data": {
            "type": "string",
            "description": "JSON row array. Historical rows include date and target; future-known rows may omit target."
        },
        "file_path": {
            "type": "string",
            "description": "CSV or Excel path used instead of data."
        },
        "data_profile_id": {
            "type": "string",
            "description": "Reusable id returned by forecast_data_audit. Use it instead of data or file_path."
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
        "frequency": {"type": "string", "description": "Frequency such as D, B, W, M, Q, Y, H, or T."},
        "confidence_level": {"type": "number", "description": "Prediction interval confidence, usually 0.9."}
    }))
    .unwrap_or_default()
}
