use super::input_data::parse_request_input;
use super::types::ForecastRequest;

fn make_request(data: &str) -> ForecastRequest {
    ForecastRequest {
        data: Some(data.to_string()),
        file_path: None,
        target_column: "sales".into(),
        date_column: "date".into(),
        covariate_columns: vec!["weather".into()],
        horizon: 3,
        frequency: "D".into(),
        model: Some("chronos-bolt-small".into()),
        confidence_level: 0.9,
    }
}

#[test]
fn parses_history_and_builds_future_dates() {
    let request = make_request(
        r#"
        [
          {"date":"2026-05-01","sales":100,"weather":"sunny"},
          {"date":"2026-05-02","sales":120,"weather":"rain"},
          {"date":"2026-05-03","sales":"135.5","weather":"wind"}
        ]
        "#,
    );

    let parsed = parse_request_input(&request).expect("input should parse");

    assert_eq!(parsed.summary.points, 3);
    assert_eq!(parsed.summary.start, "2026-05-01");
    assert_eq!(parsed.summary.end, "2026-05-03");
    assert_eq!(parsed.values, vec![100.0, 120.0, 135.5]);
    assert_eq!(parsed.snapshot.history.len(), 3);
    assert_eq!(parsed.future_dates, vec!["2026-05-04", "2026-05-05", "2026-05-06"]);
    assert!(parsed.snapshot.columns.iter().any(|column| column == "weather"));
}

#[test]
fn rejects_missing_covariate_column() {
    let mut request = make_request(
        r#"
        [
          {"date":"2026-05-01","sales":100},
          {"date":"2026-05-02","sales":120}
        ]
        "#,
    );
    request.covariate_columns = vec!["holiday".into()];

    let error = parse_request_input(&request).expect_err("missing covariate should fail");

    assert_eq!(error, "Covariable introuvable");
}

#[test]
fn falls_back_to_relative_dates_for_unknown_formats() {
    let mut request = make_request(
        r#"
        [
          {"date":"week-1","sales":100,"weather":"sunny"},
          {"date":"week-2","sales":120,"weather":"rain"}
        ]
        "#,
    );
    request.frequency = "W".into();

    let parsed = parse_request_input(&request).expect("input should parse");

    assert_eq!(parsed.future_dates, vec!["T+1", "T+2", "T+3"]);
}
