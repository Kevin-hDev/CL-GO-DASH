use std::cmp::Ordering;

use super::{AutoCandidate, CandidateBacktest, ResourceFit};

pub(super) fn compare(left: &AutoCandidate, right: &AutoCandidate, rolling: bool) -> Ordering {
    if rolling {
        match (&left.backtest, &right.backtest) {
            (Some(left), Some(right)) => return compare_backtests(left, right),
            (Some(_), None) => return Ordering::Less,
            (None, Some(_)) => return Ordering::Greater,
            (None, None) => {}
        }
    }
    resource_rank(right.resource_fit)
        .cmp(&resource_rank(left.resource_fit))
        .then_with(|| left.estimated_ram_mb.cmp(&right.estimated_ram_mb))
        .then_with(|| left.model_id.cmp(&right.model_id))
}

fn compare_backtests(left: &CandidateBacktest, right: &CandidateBacktest) -> Ordering {
    baseline_rank(right.beats_best_baseline)
        .cmp(&baseline_rank(left.beats_best_baseline))
        .then_with(|| left.metrics.mase.total_cmp(&right.metrics.mase))
        .then_with(|| left.metrics.smape.total_cmp(&right.metrics.smape))
        .then_with(|| left.metrics.mae.total_cmp(&right.metrics.mae))
        .then_with(|| left.metrics.rmse.total_cmp(&right.metrics.rmse))
        .then_with(|| left.metrics.bias.abs().total_cmp(&right.metrics.bias.abs()))
        .then_with(|| coverage_gap(left).total_cmp(&coverage_gap(right)))
        .then_with(|| left.metrics.stability.total_cmp(&right.metrics.stability))
        .then_with(|| left.duration_ms.cmp(&right.duration_ms))
}

fn baseline_rank(value: Option<bool>) -> u8 {
    match value {
        Some(true) => 2,
        None => 1,
        Some(false) => 0,
    }
}

fn coverage_gap(backtest: &CandidateBacktest) -> f64 {
    backtest
        .calibration
        .as_ref()
        .map_or(f64::INFINITY, |value| {
            (value.measured_coverage - value.theoretical_coverage).abs()
        })
}

fn resource_rank(fit: ResourceFit) -> u8 {
    match fit {
        ResourceFit::Comfortable => 4,
        ResourceFit::Cloud => 3,
        ResourceFit::Constrained => 2,
        ResourceFit::Unknown => 1,
        ResourceFit::Insufficient => 0,
    }
}
