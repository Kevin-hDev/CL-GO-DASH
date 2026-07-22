use super::*;

fn observation(actual: f64, predicted: f64, lower: f64, upper: f64, fold: usize) -> Observation {
    Observation {
        actual,
        predicted,
        lower,
        upper,
        scale: 1.0,
        fold,
    }
}

#[test]
fn metrics_are_finite_with_zero_targets() {
    let observations = vec![
        observation(0.0, 0.0, -1.0, 1.0, 0),
        observation(2.0, 1.0, 0.0, 2.0, 1),
    ];
    let result = summarize(&observations).expect("metrics");
    assert!(result.smape.is_finite());
    assert_eq!(result.mae, 0.5);
}

#[test]
fn calibration_reports_measured_coverage() {
    let observations = vec![
        observation(1.0, 1.0, 0.0, 2.0, 0),
        observation(4.0, 1.0, 0.0, 2.0, 0),
    ];
    let result = calibration(&observations, 0.9).expect("calibration");
    assert_eq!(result.measured_coverage, 0.5);
    assert_eq!(result.residual_half_width, 3.0);
}
