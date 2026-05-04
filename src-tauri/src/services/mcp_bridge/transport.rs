use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

pub static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

pub const MAX_TOOLS: usize = 128;
pub const MAX_DESC_CHARS: usize = 250;
pub const MAX_NAME_CHARS: usize = 64;

#[derive(Clone, Deserialize)]
pub struct McpToolDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

pub fn sanitize_tool_def(tool: &mut McpToolDef) {
    tool.name = tool.name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(MAX_NAME_CHARS)
        .collect();
    if let Some(ref desc) = tool.description {
        let sanitized: String = desc.chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .take(MAX_DESC_CHARS)
            .collect();
        tool.description = Some(sanitized);
    }
    if let Some(ref schema) = tool.input_schema {
        if json_depth(schema) > 4 || json_property_count(schema) > 20 {
            tool.input_schema = None;
        }
    }
}

fn json_depth(val: &Value) -> usize {
    match val {
        Value::Object(map) => 1 + map.values().map(json_depth).max().unwrap_or(0),
        Value::Array(arr) => 1 + arr.iter().map(json_depth).max().unwrap_or(0),
        _ => 0,
    }
}

fn json_property_count(val: &Value) -> usize {
    match val {
        Value::Object(map) => {
            let own = map.len();
            let nested: usize = map.values().map(json_property_count).sum();
            own + nested
        }
        Value::Array(arr) => arr.iter().map(json_property_count).sum(),
        _ => 0,
    }
}

pub fn sanitize_tools(tools: Vec<McpToolDef>) -> Vec<McpToolDef> {
    tools.into_iter()
        .take(MAX_TOOLS)
        .map(|mut t| { sanitize_tool_def(&mut t); t })
        .filter(|t| !t.name.is_empty())
        .collect()
}

pub fn extract_tool_result(resp: &Value) -> Result<String, String> {
    if let Some(err) = resp.get("error") {
        let msg = err["message"].as_str().unwrap_or("erreur inconnue");
        return Err(format!("erreur MCP : {msg}"));
    }

    let result = resp.get("result").ok_or("réponse vide du serveur MCP")?;

    if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
        let texts: Vec<&str> = content
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
            .collect();
        if !texts.is_empty() {
            return Ok(texts.join("\n"));
        }
    }

    Ok(serde_json::to_string_pretty(result).unwrap_or_default())
}

#[async_trait]
pub trait McpTransport: Send + Sync {
    async fn list_tools(&self) -> Result<Vec<McpToolDef>, String>;
    async fn call_tool(&self, name: &str, args: Value) -> Result<String, String>;
    fn transport_type(&self) -> &'static str;
}
