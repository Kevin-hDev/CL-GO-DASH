use crate::services::oauth_providers::ProviderId;
use serde_json::{json, Value};

const SHARED: &[&str] = &[
    "search_mcp_tools",
    "forecast",
    "forecast_models",
    "forecast_analyze",
    "forecast_read",
    "read_spreadsheet",
    "read_document",
    "read_image",
    "write_spreadsheet",
    "write_document",
    "process_image",
];
const GROK: &[&str] = &[
    "search_mcp_tools",
    "forecast",
    "forecast_models",
    "forecast_analyze",
    "forecast_read",
    "read_spreadsheet",
    "read_document",
    "read_image",
    "write_spreadsheet",
    "write_document",
    "process_image",
    "bash",
];

pub fn allowed_names(provider: ProviderId) -> &'static [&'static str] {
    match provider {
        ProviderId::Moonshot => SHARED,
        ProviderId::Xai => GROK,
        ProviderId::OpenAi => &[],
    }
}

pub fn contains(provider: ProviderId, name: &str) -> bool {
    allowed_names(provider).contains(&name)
}

pub async fn definitions(provider: ProviderId) -> Vec<Value> {
    let mut result = Vec::new();
    for definition in crate::services::agent_local::tool_dispatcher::get_tool_definitions() {
        let Some(function) = definition.get("function") else {
            continue;
        };
        let Some(name) = function.get("name").and_then(Value::as_str) else {
            continue;
        };
        if !contains(provider, name) {
            continue;
        }
        if crate::services::agent_local::tool_catalog::is_optional_tool(name)
            && !crate::services::agent_local::agent_settings::is_tool_enabled(name).await
        {
            continue;
        }
        result.push(json!({
            "name": name,
            "description": function.get("description").cloned().unwrap_or_default(),
            "inputSchema": function.get("parameters").cloned().unwrap_or_else(|| json!({"type":"object"})),
        }));
    }
    result
}
