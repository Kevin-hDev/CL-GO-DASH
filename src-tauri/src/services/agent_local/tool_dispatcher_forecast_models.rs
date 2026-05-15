use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::{model_listing, selected_model};
use serde_json::Value;

pub fn handle() -> ToolResult {
    let listing = model_listing::list_models();
    let Some(models) = listing["models"].as_array() else {
        return ToolResult::err("Catalogue Forecast indisponible");
    };
    let compact: Vec<Value> = models.iter().filter_map(compact_model).collect();
    let forced_model = selected_model::get();
    let payload = serde_json::json!({
        "models": compact,
        "selection_policy": {
            "mode": if forced_model.is_some() { "selector_forced" } else { "missing_selection" },
            "forced_model": forced_model,
            "forecast_model_argument": "not_supported",
            "selector_locked": true
        },
        "usage": "The Forecast UI selector is the source of truth. The forecast tool does not accept model choice from the LLM. Local models require installed=true. Cloud models require provider_configured=true."
    });
    match serde_json::to_string_pretty(&payload) {
        Ok(json) => ToolResult::ok(json),
        Err(_) => ToolResult::err("Catalogue Forecast indisponible"),
    }
}

fn compact_model(model: &Value) -> Option<Value> {
    Some(serde_json::json!({
        "id": model["id"].as_str()?,
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
