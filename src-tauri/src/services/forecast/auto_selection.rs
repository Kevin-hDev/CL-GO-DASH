use serde::Serialize;
use serde_json::Value;

use super::data_quality::DataProfile;
use super::evaluation::types::{BacktestIndexResult, BacktestIndexSummary, BacktestMetrics};
use super::hardware_profile::{HardwareProfile, ResourceFit};
use super::{catalog, limits, registry, validation};

#[path = "auto_selection_rank.rs"]
mod rank;

const MAX_UNKNOWN_RESOURCE_RAM_MB: u32 = 2_048;

#[derive(Debug, Clone, Serialize)]
pub struct AutoCandidate {
    pub model_id: String,
    pub compatibility: &'static str,
    pub resource_fit: ResourceFit,
    pub reasons: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtest: Option<CandidateBacktest>,
    #[serde(skip)]
    pub evidence: Option<BacktestIndexSummary>,
    #[serde(skip)]
    pub(crate) estimated_ram_mb: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateBacktest {
    pub evaluated_at: String,
    pub windows: usize,
    pub metrics: BacktestMetrics,
    pub duration_ms: u64,
    pub beats_best_baseline: Option<bool>,
    pub calibration: Option<super::evaluation::types::IntervalCalibration>,
}

pub struct AutoSelection {
    pub candidates: Vec<AutoCandidate>,
    pub basis: &'static str,
}

pub fn select(
    models: &[Value],
    profile: &DataProfile,
    cloud_allowed: bool,
    hardware: HardwareProfile,
    evidence: &[BacktestIndexSummary],
) -> AutoSelection {
    let mut candidates: Vec<_> = models
        .iter()
        .filter_map(|model| candidate(model, profile, cloud_allowed, hardware, evidence))
        .collect();
    let rolling = candidates
        .iter()
        .any(|candidate| candidate.backtest.is_some());
    candidates.sort_by(|left, right| rank::compare(left, right, rolling));
    candidates.truncate(limits::MAX_AUTO_CANDIDATES);
    if let Some(first) = candidates.first_mut().filter(|candidate| {
        !rolling
            || candidate
                .backtest
                .as_ref()
                .and_then(|backtest| backtest.beats_best_baseline)
                == Some(true)
    }) {
        first.compatibility = "recommended";
    }
    AutoSelection {
        candidates,
        basis: if rolling {
            "rolling_backtest"
        } else {
            "capabilities_and_resources"
        },
    }
}

fn candidate(
    model: &Value,
    profile: &DataProfile,
    cloud_allowed: bool,
    hardware: HardwareProfile,
    evidence: &[BacktestIndexSummary],
) -> Option<AutoCandidate> {
    let id = model["id"].as_str()?;
    let spec = catalog::find_model(id)?;
    let runtime = registry::find_runtime(id)?;
    if !model["runnable"].as_bool().unwrap_or(false)
        || !model["runtime_ready"].as_bool().unwrap_or(false)
        || (spec.is_cloud && !cloud_allowed)
        || !task_compatible(spec, runtime, profile)
    {
        return None;
    }
    let resource_fit = super::hardware_profile::resource_fit(spec, hardware);
    if resource_fit == ResourceFit::Insufficient
        || (resource_fit == ResourceFit::Unknown && spec.ram_mb > MAX_UNKNOWN_RESOURCE_RAM_MB)
    {
        return None;
    }
    let full_evidence = evidence_for(id, evidence);
    let backtest = full_evidence
        .as_ref()
        .and_then(|summary| compact_backtest(id, summary));
    let mut reasons = reasons(profile, resource_fit);
    if backtest.is_some() {
        reasons.push("backtest_available");
    }
    if backtest.as_ref().and_then(|item| item.beats_best_baseline) == Some(true) {
        reasons.push("beats_best_baseline");
    }
    reasons.truncate(limits::MAX_AUTO_REASONS);
    Some(AutoCandidate {
        model_id: id.to_string(),
        compatibility: "compatible",
        resource_fit,
        reasons,
        backtest,
        evidence: full_evidence,
        estimated_ram_mb: spec.ram_mb,
    })
}

fn evidence_for(model_id: &str, evidence: &[BacktestIndexSummary]) -> Option<BacktestIndexSummary> {
    evidence
        .iter()
        .find(|summary| {
            let baseline_available = summary.results.iter().any(|result| {
                result.kind == super::evaluation::types::BacktestKind::Baseline
                    && result.metrics.is_some()
            });
            baseline_available
                && summary.results.iter().any(|result| {
                    result.model_id == model_id
                        && result.kind == super::evaluation::types::BacktestKind::Model
                        && result.metrics.is_some()
                })
        })
        .cloned()
}

fn compact_backtest(model_id: &str, summary: &BacktestIndexSummary) -> Option<CandidateBacktest> {
    let result: &BacktestIndexResult = summary.results.iter().find(|result| {
        result.model_id == model_id && result.kind == super::evaluation::types::BacktestKind::Model
    })?;
    Some(CandidateBacktest {
        evaluated_at: summary.created_at.clone(),
        windows: summary.windows,
        metrics: result.metrics.clone()?,
        duration_ms: result.duration_ms,
        beats_best_baseline: result.beats_best_baseline,
        calibration: result.calibration.clone(),
    })
}

fn task_compatible(
    spec: &catalog::ForecastModelSpec,
    runtime: &registry::ForecastRuntimeSpec,
    profile: &DataProfile,
) -> bool {
    let needs_covariates = !profile.covariate_columns.is_empty();
    let needs_future = needs_covariates && profile.future_rows > 0;
    validation::effective_horizon_max(spec.id, spec.horizon_max)
        .is_ok_and(|maximum| profile.horizon <= maximum)
        && validation::supports_frequency(spec, &profile.frequency)
        && (profile.series_count <= 1 || runtime.capabilities.multivariate)
        && (!needs_covariates || runtime.capabilities.past_covariates)
        && (!needs_future || runtime.capabilities.future_covariates)
        && runtime.capabilities.probabilistic
}

fn reasons(profile: &DataProfile, fit: ResourceFit) -> Vec<&'static str> {
    let mut reasons = vec!["horizon_supported", "frequency_supported"];
    if profile.series_count > 1 {
        reasons.push("multi_series_supported");
    }
    if !profile.covariate_columns.is_empty() {
        reasons.push("covariates_supported");
    }
    if profile.future_rows > 0 && !profile.covariate_columns.is_empty() {
        reasons.push("future_covariates_supported");
    }
    if fit != ResourceFit::Unknown {
        reasons.push("resources_checked");
    }
    reasons
}
