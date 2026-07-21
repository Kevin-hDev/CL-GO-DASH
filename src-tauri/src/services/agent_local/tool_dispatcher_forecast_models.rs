use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::{model_listing, selection_policy};
use serde_json::Value;

pub fn handle() -> ToolResult {
    let listing = model_listing::list_models();
    let Some(models) = listing["models"].as_array() else {
        return ToolResult::err("Catalogue Forecast indisponible");
    };
    let policy = match selection_policy::get() {
        Ok(policy) => policy,
        Err(error) => return ToolResult::err(error),
    };
    let forced_model = (policy.mode == selection_policy::ForecastSelectionMode::Manual)
        .then_some(policy.manual_model_id.as_deref())
        .flatten();
    let mut compact: Vec<Value> = models
        .iter()
        .filter_map(|model| compact_model(model, forced_model))
        .collect();
    compact.sort_by_key(model_sort_key);
    let forced_model_state = forced_model
        .and_then(|id| models.iter().find(|model| model["id"].as_str() == Some(id)))
        .and_then(|model| compact_model(model, forced_model));
    let installed_model_ids: Vec<&str> = compact
        .iter()
        .filter(|model| model["installed"].as_bool().unwrap_or(false))
        .filter_map(|model| model["id"].as_str())
        .collect();
    let runnable_model_ids: Vec<&str> = compact
        .iter()
        .filter(|model| model["runnable"].as_bool().unwrap_or(false))
        .filter_map(|model| model["id"].as_str())
        .collect();
    let payload = match policy.mode {
        selection_policy::ForecastSelectionMode::Manual => serde_json::json!({
            "selection_policy": {
                "mode": "manual",
                "forced_model": forced_model,
                "forced_model_state": forced_model_state
            },
            "summary": {
                "installed_model_ids": installed_model_ids,
                "runnable_model_ids": runnable_model_ids
            },
            "models": compact,
            "usage": "You must use the forced model and keep the user's policy unchanged."
        }),
        selection_policy::ForecastSelectionMode::Auto => {
            let candidates = auto_candidates(&compact, policy.allow_cloud_in_auto);
            serde_json::json!({
                "selection_policy": {
                    "mode": "auto",
                    "cloud_allowed": policy.allow_cloud_in_auto
                },
                "candidates": candidates,
                "selection_basis": "capabilities_only",
                "usage": "You must choose one candidate id for forecast. This initial ranking has no comparable backtest yet, so do not call it the best model."
            })
        }
    };
    match serde_json::to_string_pretty(&payload) {
        Ok(json) => ToolResult::ok(json),
        Err(_) => ToolResult::err("Catalogue Forecast indisponible"),
    }
}

fn auto_candidates(models: &[Value], cloud_allowed: bool) -> Vec<Value> {
    models
        .iter()
        .filter(|model| model["runnable"].as_bool().unwrap_or(false))
        .filter(|model| {
            let is_cloud = model["is_cloud"].as_bool().unwrap_or(false);
            !is_cloud || cloud_allowed
        })
        .take(crate::services::forecast::limits::MAX_AUTO_CANDIDATES)
        .map(|model| {
            let is_cloud = model["is_cloud"].as_bool().unwrap_or(false);
            serde_json::json!({
                "model_id": model["id"],
                "compatibility": "compatible",
                "resource_fit": if is_cloud { "cloud" } else { "unknown" },
                "reasons": if is_cloud { vec!["cloud_allowed"] } else { vec!["local_runnable"] }
            })
        })
        .collect()
}

fn compact_model(model: &Value, forced_model: Option<&str>) -> Option<Value> {
    let id = model["id"].as_str()?;
    Some(serde_json::json!({
        "id": id,
        "selected": forced_model == Some(id),
        "name": model["display_name"].as_str().unwrap_or(""),
        "provider": model["provider_id"].as_str().unwrap_or(""),
        "family": model["family_id"].as_str().unwrap_or(""),
        "installed": model["installed"].as_bool().unwrap_or(false),
        "runnable": model["runnable"].as_bool().unwrap_or(false),
        "runtime_ready": model["runtime_ready"].as_bool().unwrap_or(false),
        "provider_configured": model["provider_configured"].as_bool().unwrap_or(false),
        "is_cloud": model["is_cloud"].as_bool().unwrap_or(false),
        "interval_support": crate::services::forecast::validation::interval_support(id),
        "capabilities": model["capabilities"].clone()
    }))
}

fn model_sort_key(model: &Value) -> (bool, bool, bool, String) {
    (
        !model["selected"].as_bool().unwrap_or(false),
        !model["runnable"].as_bool().unwrap_or(false),
        !model["installed"].as_bool().unwrap_or(false),
        model["id"].as_str().unwrap_or_default().to_string(),
    )
}

#[cfg(test)]
#[path = "tool_dispatcher_forecast_models_tests.rs"]
mod tests;
