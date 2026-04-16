//! Client Firecrawl — extraction Markdown LLM-ready d'une URL ou recherche.
//!
//! Endpoints :
//! - POST https://api.firecrawl.dev/v2/search : recherche + extraction
//! - POST https://api.firecrawl.dev/v2/scrape : extraction d'une URL unique
//!
//! Auth : `Authorization: Bearer {key}`

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_keys;
use reqwest::Client;
use std::time::Duration;

const SEARCH_URL: &str = "https://api.firecrawl.dev/v2/search";
const CREDIT_URL: &str = "https://api.firecrawl.dev/v2/team/credit-usage";
const MAX_RESULTS: usize = 10;
const TIMEOUT: Duration = Duration::from_secs(30);

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let key = api_keys::get_key("firecrawl")?;
    let client = Client::new();

    let payload = serde_json::json!({
        "query": query,
        "limit": MAX_RESULTS,
    });

    let resp = client
        .post(SEARCH_URL)
        .bearer_auth(&*key)
        .json(&payload)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("Firecrawl: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Firecrawl: HTTP {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Firecrawl parse: {e}"))?;

    let results = json["data"]["web"]
        .as_array()
        .or_else(|| json["data"].as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .take(MAX_RESULTS)
        .map(|r| SearchResult {
            title: r["title"].as_str().unwrap_or("").to_string(),
            url: r["url"].as_str().unwrap_or("").to_string(),
            snippet: r["description"]
                .as_str()
                .unwrap_or_else(|| r["markdown"].as_str().unwrap_or(""))
                .chars()
                .take(300)
                .collect(),
        })
        .collect();
    Ok(results)
}

pub async fn test_connection() -> Result<(), String> {
    let key = api_keys::get_key("firecrawl")?;
    let client = Client::new();

    let resp = client
        .get(CREDIT_URL)
        .bearer_auth(&*key)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Firecrawl: {e}"))?;

    if resp.status().is_success() {
        Ok(())
    } else if resp.status().as_u16() == 401 {
        Err("Clé Firecrawl invalide".into())
    } else {
        Err(format!("Firecrawl: HTTP {}", resp.status()))
    }
}
