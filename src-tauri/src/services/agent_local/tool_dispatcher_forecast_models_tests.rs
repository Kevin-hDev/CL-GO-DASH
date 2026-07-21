use crate::services::agent_local::tool_dispatcher_forecast_candidates::select;
use crate::services::forecast::data_quality::DataProfile;
use crate::services::forecast::hardware_profile::HardwareProfile;
use std::collections::BTreeMap;

fn model(id: &str, runnable: bool) -> serde_json::Value {
    serde_json::json!({"id": id, "runnable": runnable})
}

fn profile(series_count: usize, covariates: bool, future_rows: usize) -> DataProfile {
    DataProfile {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        created_at: "2026-01-01T00:00:00Z".into(),
        valid: true,
        target_column: "value".into(),
        date_column: "date".into(),
        series_column: (series_count > 1).then(|| "series".into()),
        covariate_columns: if covariates {
            vec!["price".into()]
        } else {
            Vec::new()
        },
        frequency: "D".into(),
        horizon: 12,
        row_count: 100,
        history_points: 100,
        future_rows,
        series_count,
        series_ids: Vec::new(),
        history_points_by_series: BTreeMap::new(),
        start: "2025-01-01".into(),
        end: "2025-12-31".into(),
        missing_periods: 0,
        outlier_count: 0,
        issues: Vec::new(),
    }
}

fn hardware() -> HardwareProfile {
    HardwareProfile {
        vram_total_mb: Some(64_000),
        vram_available_mb: Some(64_000),
        ram_available_mb: Some(64_000),
    }
}

#[test]
fn auto_candidates_are_bounded_to_five() {
    let models = [
        "chronos-bolt-tiny",
        "chronos-bolt-mini",
        "chronos-bolt-small",
        "chronos-bolt-base",
        "chronos-2",
        "timesfm-2.5-200m",
        "toto-2.0-4m",
    ]
    .map(|id| model(id, true));

    assert_eq!(select(&models, &profile(1, false, 0), false, hardware()).len(), 5);
}

#[test]
fn auto_excludes_cloud_and_non_runnable_models() {
    let models = [
        model("chronos-bolt-tiny", true),
        model("timegpt-2-mini", true),
        model("chronos-bolt-mini", false),
    ];
    let candidates = select(&models, &profile(1, false, 0), false, hardware());

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["model_id"], "chronos-bolt-tiny");
}

#[test]
fn auto_filters_against_task_capabilities() {
    let models = [
        model("chronos-bolt-tiny", true),
        model("moirai-2.0-r-small", true),
        model("chronos-2", true),
    ];
    let candidates = select(&models, &profile(2, true, 12), false, hardware());

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["model_id"], "chronos-2");
}
