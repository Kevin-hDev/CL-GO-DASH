use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::{model_listing, selected_model};
use serde_json::Value;

pub fn handle() -> ToolResult {
    let listing = model_listing::list_models();
    let Some(models) = listing["models"].as_array() else {
        return ToolResult::err("Catalogue Forecast indisponible");
    };
    let forced_model = selected_model::get();
    let mut compact: Vec<Value> = models
        .iter()
        .filter_map(|model| compact_model(model, forced_model.as_deref()))
        .collect();
    compact.sort_by_key(model_sort_key);
    let forced_model_state = forced_model
        .as_ref()
        .and_then(|id| models.iter().find(|model| model["id"].as_str() == Some(id)))
        .and_then(|model| compact_model(model, forced_model.as_deref()));
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
    let payload = serde_json::json!({
        "selection_policy": {
            "mode": if forced_model.is_some() { "selector_forced" } else { "missing_selection" },
            "forced_model": forced_model,
            "forced_model_state": forced_model_state,
            "forecast_model_argument": "not_supported",
            "selector_locked": true
        },
        "summary": {
            "installed_model_ids": installed_model_ids,
            "runnable_model_ids": runnable_model_ids
        },
        "models": compact,
        "usage": "The Forecast UI selector is the source of truth. The forecast tool does not accept model choice from the LLM. Local models require installed=true. Cloud models require provider_configured=true."
    });
    match serde_json::to_string_pretty(&payload) {
        Ok(json) => ToolResult::ok(json),
        Err(_) => ToolResult::err("Catalogue Forecast indisponible"),
    }
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
