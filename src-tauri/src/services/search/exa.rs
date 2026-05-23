//! Client Exa (ex-Metaphor) — neural search par similarité sémantique.
//!
//! Endpoint : POST https://api.exa.ai/search
//! Auth : header `x-api-key`

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_keys;
use crate::services::search::common;
use reqwest::Client;
use std::time::Duration;

const URL: &str = "https://api.exa.ai/search";
const TIMEOUT: Duration = Duration::from_secs(15);

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let query = common::validate_query(query)?;
    let key = api_keys::get_key("exa")?;
    let client = Client::new();

    let payload = serde_json::json!({
        "query": query,
        "numResults": common::MAX_RESULTS,
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

    let json = common::read_json_bounded(resp, "Exa").await?;

    let results = json["results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| {
            common::make_result(
                r["title"].as_str().unwrap_or(""),
                r["url"].as_str().unwrap_or(""),
                r["text"]
                    .as_str()
                    .unwrap_or_else(|| r["summary"].as_str().unwrap_or("")),
            )
        })
        .take(common::MAX_RESULTS)
        .collect();
    Ok(results)
}

pub async fn test_connection() -> Result<(), String> {
    let _ = search("test").await?;
    Ok(())
}
