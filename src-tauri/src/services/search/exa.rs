//! Client Exa (ex-Metaphor) — neural search par similarité sémantique.
//!
//! Endpoint : POST https://api.exa.ai/search
//! Auth : header `x-api-key`

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_key_cache;
use reqwest::Client;
use std::time::Duration;

const URL: &str = "https://api.exa.ai/search";
const MAX_RESULTS: usize = 10;
const TIMEOUT: Duration = Duration::from_secs(15);

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let key = api_key_cache::get_key("exa")?;
    let client = Client::new();

    let payload = serde_json::json!({
        "query": query,
        "numResults": MAX_RESULTS,
        "contents": {
            "text": { "maxCharacters": 500 }
        }
    });

    let resp = client
        .post(URL)
        .header("x-api-key", &*key)
        .json(&payload)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("Exa: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Exa: HTTP {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Exa parse: {e}"))?;

    let results = json["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .take(MAX_RESULTS)
        .map(|r| SearchResult {
            title: r["title"].as_str().unwrap_or("").to_string(),
            url: r["url"].as_str().unwrap_or("").to_string(),
            snippet: r["text"]
                .as_str()
                .unwrap_or_else(|| r["summary"].as_str().unwrap_or(""))
                .chars()
                .take(300)
                .collect(),
        })
        .collect();
    Ok(results)
}

pub async fn test_connection() -> Result<(), String> {
    let _ = search("test").await?;
    Ok(())
}
