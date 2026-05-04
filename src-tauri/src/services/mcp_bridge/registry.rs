use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::Deserialize;

use super::http::HttpTransport;
use super::stdio::StdioTransport;
use super::transport::{McpToolDef, McpTransport};

const MAX_CACHE: usize = 32;
const CACHE_TTL_SECS: u64 = 300;

struct CachedTools {
    tools: Vec<McpToolDef>,
    fetched_at: Instant,
}

static TOOL_CACHE: std::sync::LazyLock<Mutex<HashMap<String, CachedTools>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Deserialize)]
struct StoredConnector {
    id: String,
    status: String,
    enabled_in_chat: bool,
    endpoint: Option<String>,
    install_command: Option<String>,
    #[serde(default)]
    env_keys: Option<Vec<String>>,
}

pub struct EnabledConnector {
    pub id: String,
    pub transport: Arc<dyn McpTransport>,
}

pub fn get_enabled_connectors() -> Vec<EnabledConnector> {
    let path = crate::services::paths::data_dir().join("mcp-connectors.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let connectors: Vec<StoredConnector> = match serde_json::from_str(&content) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    connectors
        .into_iter()
        .filter(|c| c.status == "connected" && c.enabled_in_chat)
        .filter_map(build_connector)
        .take(MAX_CACHE)
        .collect()
}

fn is_valid_connector_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn build_connector(c: StoredConnector) -> Option<EnabledConnector> {
    if !is_valid_connector_id(&c.id) {
        return None;
    }
    if let Some(ref endpoint) = c.endpoint {
        if endpoint.starts_with("https://") {
            let transport = HttpTransport::new(c.id.clone(), endpoint.clone());
            return Some(EnabledConnector {
                id: c.id,
                transport: Arc::new(transport),
            });
        }
    }

    if let Some(ref cmd) = c.install_command {
        let env_tokens = resolve_env_tokens(c.env_keys.as_deref(), &c.id);
        let transport = StdioTransport::new(c.id.clone(), cmd.clone(), env_tokens);
        return Some(EnabledConnector {
            id: c.id,
            transport: Arc::new(transport),
        });
    }

    None
}

fn resolve_env_tokens(keys: Option<&[String]>, connector_id: &str) -> Vec<(String, String)> {
    let keys = match keys {
        Some(k) if !k.is_empty() => k,
        _ => return Vec::new(),
    };
    let mut result = Vec::new();
    for key in keys {
        let vault_key = format!("mcp_{connector_id}_{}", key.to_lowercase());
        if let Ok(val) = crate::services::api_keys::get_key(&vault_key) {
            result.push((key.clone(), val.to_string()));
        }
    }
    result
}

pub async fn get_tools(connector: &EnabledConnector) -> Result<Vec<McpToolDef>, String> {
    if let Some(cached) = get_cached(&connector.id) {
        return Ok(cached);
    }

    let tools = connector.transport.list_tools().await?;
    set_cached(&connector.id, &tools);
    Ok(tools)
}

pub fn invalidate_cache(connector_id: &str) {
    if let Ok(mut cache) = TOOL_CACHE.lock() {
        cache.remove(connector_id);
    }
}

fn get_cached(connector_id: &str) -> Option<Vec<McpToolDef>> {
    let cache = TOOL_CACHE.lock().ok()?;
    let entry = cache.get(connector_id)?;
    if entry.fetched_at.elapsed().as_secs() > CACHE_TTL_SECS {
        return None;
    }
    Some(entry.tools.clone())
}

fn set_cached(connector_id: &str, tools: &[McpToolDef]) {
    if let Ok(mut cache) = TOOL_CACHE.lock() {
        if cache.len() >= MAX_CACHE && !cache.contains_key(connector_id) {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.fetched_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }
        cache.insert(
            connector_id.to_string(),
            CachedTools {
                tools: tools.to_vec(),
                fetched_at: Instant::now(),
            },
        );
    }
}
