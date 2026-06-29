use serde_json::{Map, Value};

pub fn effective_config(model: &str) -> Map<String, Value> {
    crate::services::forecast::model_config::effective_values(model).unwrap_or_default()
}

pub fn effective_level(config: &Map<String, Value>, fallback_confidence: f64) -> u64 {
    config
        .get("level")
        .and_then(Value::as_u64)
        .unwrap_or((fallback_confidence * 100.0) as u64)
}

pub fn apply(payload: &mut Value, config: &Map<String, Value>) {
    let Some(object) = payload.as_object_mut() else {
        return;
    };
    for key in [
        "clean_ex_first",
        "finetune_steps",
        "finetune_loss",
        "finetune_depth",
        "feature_contributions",
    ] {
        if let Some(value) = config.get(key) {
            object.insert(key.to_string(), value.clone());
        }
    }
}

#[cfg(test)]
#[path = "client_nixtla_options_tests.rs"]
mod tests;
