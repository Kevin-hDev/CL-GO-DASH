use super::definition_for;
use crate::services::forecast::selection_policy::{
    ForecastSelectionMode, ForecastSelectionPolicy,
};

fn policy(mode: ForecastSelectionMode) -> ForecastSelectionPolicy {
    ForecastSelectionPolicy {
        mode,
        manual_model_id: Some("chronos-bolt-small".into()),
        allow_cloud_in_auto: false,
    }
}

#[test]
fn manual_schema_does_not_offer_model_override() {
    let definition = definition_for(policy(ForecastSelectionMode::Manual));
    let parameters = &definition["function"]["parameters"];

    assert!(parameters["properties"].get("model").is_none());
    assert!(parameters["properties"].get("data_profile_id").is_some());
    assert!(!parameters["required"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "model"));
}

#[test]
fn auto_schema_requires_a_candidate_model() {
    let definition = definition_for(policy(ForecastSelectionMode::Auto));
    let parameters = &definition["function"]["parameters"];

    assert!(parameters["properties"].get("model").is_some());
    assert!(parameters["required"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "model"));
    assert!(definition["function"]["description"]
        .as_str()
        .unwrap()
        .contains("call forecast_models"));
}
