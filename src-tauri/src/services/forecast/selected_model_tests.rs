use super::apply_policy;
use crate::services::forecast::selection_policy::{ForecastSelectionMode, ForecastSelectionPolicy};
use crate::services::forecast::types::ForecastRequest;

const MANUAL_MODEL: &str = "chronos-bolt-small";
const OTHER_MODEL: &str = "chronos-bolt-base";

fn request(model: Option<&str>) -> ForecastRequest {
    ForecastRequest {
        data: Some("[]".into()),
        file_path: None,
        target_column: "value".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: Vec::new(),
        horizon: 1,
        frequency: "D".into(),
        model: model.map(str::to_string),
        confidence_level: 0.9,
    }
}

fn policy(mode: ForecastSelectionMode) -> ForecastSelectionPolicy {
    ForecastSelectionPolicy {
        mode,
        manual_model_id: Some(MANUAL_MODEL.into()),
        allow_cloud_in_auto: false,
    }
}

#[test]
fn manual_rejects_a_different_requested_model() {
    let mut input = request(Some(OTHER_MODEL));

    assert!(apply_policy(&mut input, policy(ForecastSelectionMode::Manual)).is_err());
    assert_eq!(input.model.as_deref(), Some(OTHER_MODEL));
}

#[test]
fn auto_requires_and_keeps_the_candidate_selected_by_the_llm() {
    let mut missing = request(None);
    assert!(apply_policy(&mut missing, policy(ForecastSelectionMode::Auto)).is_err());

    let mut selected = request(Some(OTHER_MODEL));
    let effective = apply_policy(&mut selected, policy(ForecastSelectionMode::Auto)).unwrap();
    assert_eq!(effective, OTHER_MODEL);
    assert_eq!(selected.model.as_deref(), Some(OTHER_MODEL));
}

#[test]
fn auto_rejects_cloud_when_the_user_did_not_allow_it() {
    let mut selected = request(Some("timegpt-2-mini"));

    assert!(apply_policy(&mut selected, policy(ForecastSelectionMode::Auto)).is_err());
}
