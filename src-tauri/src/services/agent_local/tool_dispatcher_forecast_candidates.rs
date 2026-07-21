use crate::services::forecast::{
    catalog, data_quality::DataProfile, hardware_profile, limits, registry, validation,
};
use serde_json::Value;

pub fn select(
    models: &[Value],
    profile: &DataProfile,
    cloud_allowed: bool,
    hardware: hardware_profile::HardwareProfile,
) -> Vec<Value> {
    models
        .iter()
        .filter_map(|model| candidate(model, profile, cloud_allowed, hardware))
        .take(limits::MAX_AUTO_CANDIDATES)
        .collect()
}

fn candidate(
    model: &Value,
    profile: &DataProfile,
    cloud_allowed: bool,
    hardware: hardware_profile::HardwareProfile,
) -> Option<Value> {
    let id = model["id"].as_str()?;
    let spec = catalog::find_model(id)?;
    let runtime = registry::find_runtime(id)?;
    if !model["runnable"].as_bool().unwrap_or(false)
        || (spec.is_cloud && !cloud_allowed)
        || !task_compatible(spec, runtime, profile)
    {
        return None;
    }
    let resource_fit = hardware_profile::resource_fit(spec, hardware);
    if resource_fit == hardware_profile::ResourceFit::Insufficient {
        return None;
    }
    Some(serde_json::json!({
        "model_id": id,
        "compatibility": "compatible",
        "resource_fit": resource_fit,
        "reasons": reasons(profile, resource_fit)
    }))
}

fn task_compatible(
    spec: &catalog::ForecastModelSpec,
    runtime: &registry::ForecastRuntimeSpec,
    profile: &DataProfile,
) -> bool {
    let needs_covariates = !profile.covariate_columns.is_empty();
    let needs_future_covariates = needs_covariates && profile.future_rows > 0;
    let horizon_ok = validation::effective_horizon_max(spec.id, spec.horizon_max)
        .is_ok_and(|maximum| profile.horizon <= maximum);
    horizon_ok
        && validation::supports_frequency(spec, &profile.frequency)
        && (profile.series_count <= 1 || runtime.capabilities.multivariate)
        && (!needs_covariates || runtime.capabilities.past_covariates)
        && (!needs_future_covariates || runtime.capabilities.future_covariates)
        && runtime.capabilities.probabilistic
}

fn reasons(
    profile: &DataProfile,
    resource_fit: hardware_profile::ResourceFit,
) -> Vec<&'static str> {
    let mut reasons = Vec::with_capacity(limits::MAX_AUTO_REASONS);
    reasons.push("horizon_supported");
    reasons.push("frequency_supported");
    if profile.series_count > 1 {
        reasons.push("multi_series_supported");
    }
    if !profile.covariate_columns.is_empty() {
        reasons.push("covariates_supported");
    }
    if profile.future_rows > 0 && !profile.covariate_columns.is_empty() {
        reasons.push("future_covariates_supported");
    }
    if resource_fit != hardware_profile::ResourceFit::Unknown {
        reasons.push("resources_checked");
    }
    reasons.truncate(limits::MAX_AUTO_REASONS);
    reasons
}
