use super::baselines::{seasonal_period, Baseline};
use super::types::{BacktestKind, BacktestRequest, ForecastEvaluation, ModelBacktestResult};
use crate::services::forecast::limits::MAX_BACKTEST_MODELS;
use crate::services::forecast::sidecar::ChronosSidecar;
use crate::services::forecast::types::ForecastResult;
use tokio::sync::Semaphore;

static BACKTEST_LIMIT: Semaphore = Semaphore::const_new(1);

pub async fn run(
    request: BacktestRequest,
    chronos: &ChronosSidecar,
) -> Result<ForecastResult, String> {
    let _permit = BACKTEST_LIMIT
        .try_acquire()
        .map_err(|_| "Un backtest Forecast est déjà en cours".to_string())?;
    validate_request(&request)?;
    let mut analysis = crate::services::forecast::storage::load(&request.analysis_id).await?;
    let plan = super::folds::build(&analysis, request.max_windows)?;
    let period = seasonal_period(&analysis.frequency);
    let mut results: Vec<_> = Baseline::ALL
        .into_iter()
        .map(|baseline| {
            super::baseline_runner::evaluate(baseline, &plan, period, analysis.confidence_level)
        })
        .collect();
    for model_id in model_ids(&request, &analysis)? {
        results.push(super::model_runner::evaluate(&analysis, &model_id, &plan, chronos).await);
    }
    if results.len() > super::super::limits::MAX_BACKTEST_RESULTS {
        return Err("Trop de résultats de backtest".into());
    }
    rank(&mut results);
    apply_calibration(&mut analysis, &results);
    analysis.evaluation = Some(ForecastEvaluation {
        schema_version: 1,
        created_at: chrono::Utc::now().to_rfc3339(),
        horizon: plan.horizon,
        windows: plan.folds.len(),
        warning: plan.warning,
        results,
    });
    crate::services::forecast::storage::save(&mut analysis).await?;
    Ok(analysis)
}

fn validate_request(request: &BacktestRequest) -> Result<(), String> {
    if request.model_ids.len() > MAX_BACKTEST_MODELS {
        return Err("Trop de modèles à évaluer".into());
    }
    if request
        .max_windows
        .is_some_and(|value| value == 0 || value > super::super::limits::MAX_BACKTEST_WINDOWS)
    {
        return Err("Nombre de fenêtres invalide".into());
    }
    Ok(())
}

fn model_ids(request: &BacktestRequest, analysis: &ForecastResult) -> Result<Vec<String>, String> {
    let requested = if request.model_ids.is_empty() {
        vec![analysis.model.clone()]
    } else {
        request.model_ids.clone()
    };
    let mut unique = Vec::new();
    for id in requested {
        let trimmed = id.trim();
        if trimmed.is_empty()
            || trimmed.chars().count() > super::super::limits::MAX_MODEL_ID_CHARS
            || crate::services::forecast::registry::find_runtime(trimmed).is_none()
        {
            return Err("Modèle de backtest invalide".into());
        }
        if !unique.iter().any(|existing| existing == trimmed) {
            unique.push(trimmed.to_string());
        }
    }
    if unique.len() > MAX_BACKTEST_MODELS {
        return Err("Trop de modèles à évaluer".into());
    }
    Ok(unique)
}

fn rank(results: &mut [ModelBacktestResult]) {
    let best_baseline = results
        .iter()
        .filter(|result| result.kind == BacktestKind::Baseline)
        .filter_map(|result| result.metrics.as_ref().map(|metrics| metrics.mase))
        .min_by(f64::total_cmp);
    let mut order: Vec<(usize, f64)> = results
        .iter()
        .enumerate()
        .filter_map(|(index, result)| result.metrics.as_ref().map(|metrics| (index, metrics.mase)))
        .collect();
    order.sort_by(|left, right| left.1.total_cmp(&right.1));
    for (rank, (index, _)) in order.into_iter().enumerate() {
        results[index].rank = Some(rank + 1);
        if results[index].kind == BacktestKind::Baseline {
            results[index].beats_best_baseline = None;
        } else {
            results[index].beats_best_baseline = best_baseline.map(|baseline| {
                results[index]
                    .metrics
                    .as_ref()
                    .is_some_and(|metrics| metrics.mase < baseline)
            });
        }
    }
}

fn apply_calibration(analysis: &mut ForecastResult, results: &[ModelBacktestResult]) {
    let Some(calibration) = results
        .iter()
        .find(|result| result.kind == BacktestKind::Model && result.model_id == analysis.model)
        .and_then(|result| result.calibration.as_ref())
        .filter(|calibration| calibration.sample_count >= 3)
    else {
        return;
    };
    let half_width = calibration.residual_half_width;
    if !half_width.is_finite() || half_width < 0.0 {
        return;
    }
    analysis.quantiles.q10 = analysis
        .predictions
        .iter()
        .enumerate()
        .map(|(index, point)| {
            analysis
                .quantiles
                .q10
                .get(index)
                .copied()
                .unwrap_or(point.value)
                .min(point.value - half_width)
        })
        .collect();
    analysis.quantiles.q50 = analysis
        .predictions
        .iter()
        .map(|point| point.value)
        .collect();
    analysis.quantiles.q90 = analysis
        .predictions
        .iter()
        .enumerate()
        .map(|(index, point)| {
            analysis
                .quantiles
                .q90
                .get(index)
                .copied()
                .unwrap_or(point.value)
                .max(point.value + half_width)
        })
        .collect();
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
