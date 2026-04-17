//! Client Brave Search API.
//!
//! Déplacé depuis `services/agent_local/tool_web_search.rs`.
//! Utilise `api_key_cache::get_key("brave")` pour charger la clé.

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_key_cache;
use reqwest::Client;
use std::time::Duration;

const URL: &str = "https://api.search.brave.com/res/v1/web/search";
const MAX_RESULTS: usize = 10;
const TIMEOUT: Duration = Duration::from_secs(10);

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let key = api_key_cache::get_key("brave")?;
    let client = Client::new();
    let resp = client
        .get(URL)
        .query(&[("q", query), ("count", &MAX_RESULTS.to_string())])
        .header("X-Subscription-Token", &*key)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("Brave: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Brave: HTTP {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Brave parse: {e}"))?;

    let results = json["web"]["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .take(MAX_RESULTS)
        .map(|r| SearchResult {
            title: r["title"].as_str().unwrap_or("").to_string(),
            url: r["url"].as_str().unwrap_or("").to_string(),
            snippet: r["description"].as_str().unwrap_or("").to_string(),
        })
        .collect();
    Ok(results)
}

pub async fn test_connection() -> Result<(), String> {
    let _ = search("test").await?;
    Ok(())
}
