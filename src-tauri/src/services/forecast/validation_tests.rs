use super::types::ForecastRequest;
use super::validation::validate_request;

fn make_request(model: &str) -> ForecastRequest {
    ForecastRequest {
        data: Some(r#"[{"date":"2026-05-01","sales":100,"temp":22}]"#.to_string()),
        file_path: None,
        target_column: "sales".into(),
        date_column: "date".into(),
        covariate_columns: vec!["temp".into()],
        horizon: 3,
        frequency: "D".into(),
        model: Some(model.into()),
        confidence_level: 0.9,
    }
}

#[test]
fn rejects_covariates_for_current_local_chronos_runtimes() {
    let error = validate_request(&make_request("chronos-2")).unwrap_err();
    assert_eq!(error, "Variables de contexte non supportées par ce moteur");
}

#[test]
fn accepts_covariates_for_timegpt() {
    assert!(validate_request(&make_request("timegpt-2-mini")).is_ok());
}
