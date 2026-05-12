use super::types::ForecastRequest;
use super::validation::validate_request;

fn make_request(model: &str) -> ForecastRequest {
    ForecastRequest {
        data: Some(r#"[{"date":"2026-05-01","sales":100,"temp":22}]"#.to_string()),
        file_path: None,
        target_column: "sales".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: vec!["temp".into()],
        horizon: 3,
        frequency: "D".into(),
        model: Some(model.into()),
        confidence_level: 0.9,
    }
}

#[test]
fn accepts_covariates_for_chronos2() {
    assert!(validate_request(&make_request("chronos-2")).is_ok());
}

#[test]
fn rejects_multiseries_for_timegpt_in_current_app() {
    let mut request = make_request("timegpt-2-mini");
    request.series_column = Some("asset_id".into());

    assert!(validate_request(&request).is_ok());
}
