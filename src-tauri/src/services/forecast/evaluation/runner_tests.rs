use super::*;
use crate::services::forecast::evaluation::types::{BacktestMetrics, IntervalCalibration};

fn result(id: &str, kind: BacktestKind, mase: f64) -> ModelBacktestResult {
    ModelBacktestResult {
        model_id: id.into(),
        kind,
        metrics: Some(BacktestMetrics {
            mase,
            smape: 1.0,
            mae: 1.0,
            rmse: 1.0,
            bias: 0.0,
            stability: 0.0,
        }),
        calibration: None,
        folds: vec![],
        duration_ms: 0,
        rank: None,
        beats_best_baseline: None,
        warning: None,
    }
}

#[test]
fn advanced_model_must_beat_the_best_baseline() {
    let mut results = vec![
        result("naive", BacktestKind::Baseline, 1.0),
        result("model", BacktestKind::Model, 0.8),
    ];
    rank(&mut results);
    assert_eq!(results[1].rank, Some(1));
    assert_eq!(results[1].beats_best_baseline, Some(true));
}

#[test]
fn calibration_widens_saved_future_intervals() {
    let mut analysis: ForecastResult = serde_json::from_value(serde_json::json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "test",
        "target_column": "y",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "model",
        "provider": "local",
        "horizon": 1,
        "frequency": "D",
        "input_summary": {"points": 4, "start": "2025-01-01", "end": "2025-01-04"},
        "predictions": [{"date": "2025-01-05", "value": 10.0}],
        "quantiles": {"q10": [9.0], "q50": [10.0], "q90": [11.0]}
    }))
    .unwrap();
    let mut model = result("model", BacktestKind::Model, 0.8);
    model.calibration = Some(IntervalCalibration {
        theoretical_coverage: 0.9,
        measured_coverage: 0.7,
        mean_width: 2.0,
        residual_half_width: 3.0,
        sample_count: 6,
    });
    apply_calibration(&mut analysis, &[model]);
    assert_eq!(analysis.quantiles.q10, [7.0]);
    assert_eq!(analysis.quantiles.q90, [13.0]);
}
