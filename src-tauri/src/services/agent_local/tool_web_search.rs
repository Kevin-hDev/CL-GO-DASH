use crate::services::agent_local::types_tools::SearchResult;
use reqwest::Client;
use std::time::Duration;

const BRAVE_URL: &str = "https://api.search.brave.com/res/v1/web/search";
const SEARXNG_URL: &str = "http://localhost:8080/search";
const MAX_RESULTS: usize = 10;
const TIMEOUT: Duration = Duration::from_secs(10);
const KEYRING_SERVICE: &str = "cl-go-dash";
const KEYRING_USER: &str = "brave_api_key";

pub async fn web_search(query: &str) -> Result<Vec<SearchResult>, String> {
    if let Ok(key) = get_brave_key() {
        return brave_search(query, &key).await;
    }
    if let Ok(results) = searxng_search(query).await {
        if !results.is_empty() {
            return Ok(results);
        }
    }
    Err("Recherche web indisponible. Configure une clé API Brave Search dans les paramètres (gratuit sur brave.com/search/api) ou lance SearXNG en local sur le port 8080.".to_string())
}

async fn brave_search(query: &str, api_key: &str) -> Result<Vec<SearchResult>, String> {
    let client = Client::new();
    let resp = client
        .get(BRAVE_URL)
        .query(&[("q", query), ("count", &MAX_RESULTS.to_string())])
        .header("X-Subscription-Token", api_key)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("Brave: {e}"))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
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

async fn searxng_search(query: &str) -> Result<Vec<SearchResult>, String> {
    let client = Client::new();
    let resp = client
        .get(SEARXNG_URL)
        .query(&[("q", query), ("format", "json")])
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("SearXNG: {e}"))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let results = json["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .take(MAX_RESULTS)
        .map(|r| SearchResult {
            title: r["title"].as_str().unwrap_or("").to_string(),
            url: r["url"].as_str().unwrap_or("").to_string(),
            snippet: r["content"].as_str().unwrap_or("").to_string(),
        })
        .collect();
    Ok(results)
}

fn get_brave_key() -> Result<String, String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

pub fn set_brave_key(key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| e.to_string())?;
    entry.set_password(key).map_err(|e| e.to_string())
}
