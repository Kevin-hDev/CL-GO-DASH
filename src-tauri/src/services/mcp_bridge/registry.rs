use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use serde::Deserialize;

use super::client::{self, McpToolDef};
use crate::services::mcp_oauth::storage;

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
}

pub struct EnabledConnector {
    pub id: String,
    pub endpoint: String,
}

pub fn get_enabled_connectors() -> Vec<EnabledConnector> {
    let path = crate::services::paths::data_dir().join("mcp-connectors.json");
    #[cfg(debug_assertions)]
    eprintln!("[mcp-registry] path: {}", path.display());
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            #[cfg(debug_assertions)]
    eprintln!("[mcp-registry] read error: {e}");
            return Vec::new();
        }
    };
    #[cfg(debug_assertions)]
    eprintln!("[mcp-registry] raw json len={}", content.len());
    let connectors: Vec<StoredConnector> = match serde_json::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            #[cfg(debug_assertions)]
    eprintln!("[mcp-registry] parse error: {e}");
            return Vec::new();
        }
    };
    #[cfg(debug_assertions)]
    eprintln!("[mcp-registry] parsed {} connectors", connectors.len());
    let result: Vec<EnabledConnector> = connectors
        .into_iter()
        .filter(|c| {
            let ok = c.status == "connected" && c.enabled_in_chat;
            #[cfg(debug_assertions)]
    eprintln!("[mcp-registry]   {} status={} chat={} endpoint={:?} → filter={ok}",
                c.id, c.status, c.enabled_in_chat, c.endpoint.as_deref().unwrap_or("NONE"));
            ok
        })
        .filter_map(|c| {
            let endpoint = c.endpoint?;
            if !endpoint.starts_with("https://") {
                return None;
            }
            Some(EnabledConnector { id: c.id, endpoint })
        })
        .take(MAX_CACHE)
        .collect();
    #[cfg(debug_assertions)]
    eprintln!("[mcp-registry] result: {} enabled connectors", result.len());
    result
}

pub async fn get_tools(connector: &EnabledConnector) -> Result<Vec<McpToolDef>, String> {
    if let Some(cached) = get_cached(&connector.id) {
        return Ok(cached);
    }

    let token = storage::get_valid_token(&connector.id).await?;
    let tools = client::list_tools(&connector.endpoint, token.as_str()).await?;

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
