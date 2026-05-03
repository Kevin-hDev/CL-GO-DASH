use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use super::response;

const TIMEOUT: Duration = Duration::from_secs(30);
const MAX_TOOLS: usize = 128;
const ACCEPT: &str = "application/json, text/event-stream";

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Deserialize)]
pub struct McpToolDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

pub async fn list_tools(endpoint: &str, token: &str) -> Result<Vec<McpToolDef>, String> {
    let session_id = initialize(endpoint, token).await?;

    let body = serde_json::json!({
        "jsonrpc": "2.0", "method": "tools/list", "id": next_id()
    });

    let resp = mcp_post(endpoint, token, session_id.as_deref(), &body).await?;

    let tools_val = resp
        .result
        .and_then(|r| r.get("tools").cloned())
        .ok_or("réponse tools/list invalide")?;

    let tools: Vec<McpToolDef> =
        serde_json::from_value(tools_val).map_err(|_| "format tools invalide")?;

    if tools.len() > MAX_TOOLS {
        return Ok(tools.into_iter().take(MAX_TOOLS).collect());
    }
    Ok(tools)
}

pub async fn call_tool(
    endpoint: &str,
    token: &str,
    tool_name: &str,
    arguments: Value,
) -> Result<String, String> {
    let session_id = initialize(endpoint, token).await?;

    let body = serde_json::json!({
        "jsonrpc": "2.0", "method": "tools/call", "id": next_id(),
        "params": { "name": tool_name, "arguments": arguments }
    });

    let resp = mcp_post(endpoint, token, session_id.as_deref(), &body).await?;

    if let Some(err) = resp.error {
        let msg = err["message"].as_str().unwrap_or("erreur inconnue");
        return Err(format!("erreur MCP : {msg}"));
    }

    let result = resp.result.ok_or("réponse vide du serveur MCP")?;

    if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
        let texts: Vec<&str> = content
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
            .collect();
        if !texts.is_empty() {
            return Ok(texts.join("\n"));
        }
    }

    Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
}

async fn initialize(endpoint: &str, token: &str) -> Result<Option<String>, String> {
    let init_body = serde_json::json!({
        "jsonrpc": "2.0", "method": "initialize", "id": next_id(),
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "CL-GO-DASH", "version": env!("CARGO_PKG_VERSION") }
        }
    });

    let client = build_client()?;

    let resp = client
        .post(endpoint)
        .bearer_auth(token)
        .header("Accept", ACCEPT)
        .header("Content-Type", "application/json")
        .json(&init_body)
        .send()
        .await
        .map_err(|_| "impossible de contacter le serveur MCP")?;

    if !resp.status().is_success() {
        return Err("le serveur MCP a refusé l'initialisation".to_string());
    }

    let session_id = resp
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let _ = response::parse(resp).await?;

    let notif = serde_json::json!({
        "jsonrpc": "2.0", "method": "notifications/initialized"
    });

    let mut req = client
        .post(endpoint)
        .bearer_auth(token)
        .header("Accept", ACCEPT)
        .header("Content-Type", "application/json");

    if let Some(ref sid) = session_id {
        req = req.header("Mcp-Session-Id", sid);
    }

    let _ = req.json(&notif).send().await;
    Ok(session_id)
}

async fn mcp_post(
    endpoint: &str,
    token: &str,
    session_id: Option<&str>,
    body: &Value,
) -> Result<response::JsonRpcResponse, String> {
    let client = build_client()?;

    let mut req = client
        .post(endpoint)
        .bearer_auth(token)
        .header("Accept", ACCEPT)
        .header("Content-Type", "application/json");

    if let Some(sid) = session_id {
        req = req.header("Mcp-Session-Id", sid);
    }

    let resp = req.json(body).send().await
        .map_err(|_| "impossible de contacter le serveur MCP")?;

    if !resp.status().is_success() {
        return Err("le serveur MCP a refusé la requête".to_string());
    }

    response::parse(resp).await
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(TIMEOUT)
        .build()
        .map_err(|_| "erreur interne".to_string())
}
