//! Module Search / Scraping multi-provider.
//!
//! Le tool `web_search` de l'agent utilise `run_search` qui route vers
//! le premier provider configuré dans l'ordre de préférence :
//! Brave → Exa → Firecrawl → SearXNG (fallback local sans clé).

pub mod brave;
pub mod catalog;
pub mod exa;
pub mod firecrawl;

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_keys;

/// Fallback local SearXNG (pas de clé, port 8080).
const SEARXNG_URL: &str = "http://localhost:8080/search";
const SEARXNG_MAX: usize = 10;

/// Orchestrateur de recherche web — essaie chaque provider dans l'ordre.
pub async fn run_search(query: &str) -> Result<Vec<SearchResult>, String> {
    // 1. Brave (si configuré)
    if api_keys::has_key("brave") {
        match brave::search(query).await {
            Ok(results) if !results.is_empty() => return Ok(results),
            Ok(_) => {}
            Err(e) => eprintln!("[search] Brave: {}", e),
        }
    }

    // 2. Exa (si configuré)
    if api_keys::has_key("exa") {
        match exa::search(query).await {
            Ok(results) if !results.is_empty() => return Ok(results),
            Ok(_) => {}
            Err(e) => eprintln!("[search] Exa: {}", e),
        }
    }

    // 3. Firecrawl (si configuré)
    if api_keys::has_key("firecrawl") {
        match firecrawl::search(query).await {
            Ok(results) if !results.is_empty() => return Ok(results),
            Ok(_) => {}
            Err(e) => eprintln!("[search] Firecrawl: {}", e),
        }
    }

    // 4. SearXNG local (fallback sans clé)
    if let Ok(results) = searxng_search(query).await {
        if !results.is_empty() {
            return Ok(results);
        }
    }

    Err("Aucun provider de recherche web configuré. Va dans l'onglet \"Clé API\" pour ajouter Brave, Exa ou Firecrawl, ou lance SearXNG en local sur le port 8080.".to_string())
}

async fn searxng_search(query: &str) -> Result<Vec<SearchResult>, String> {
    use reqwest::Client;
    use std::time::Duration;

    let client = Client::new();
    let resp = client
        .get(SEARXNG_URL)
        .query(&[("q", query), ("format", "json")])
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("SearXNG: {e}"))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let results = json["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .take(SEARXNG_MAX)
        .map(|r| SearchResult {
            title: r["title"].as_str().unwrap_or("").to_string(),
            url: r["url"].as_str().unwrap_or("").to_string(),
            snippet: r["content"].as_str().unwrap_or("").to_string(),
        })
        .collect();
    Ok(results)
}

/// Test de connexion d'un provider search spécifique (utilisé par l'UI
/// quand l'utilisateur colle une clé).
pub async fn test_connection(provider_id: &str) -> Result<(), String> {
    match provider_id {
        "brave" => brave::test_connection().await,
        "exa" => exa::test_connection().await,
        "firecrawl" => firecrawl::test_connection().await,
        other => Err(format!("Test non implémenté pour {}", other)),
    }
}
