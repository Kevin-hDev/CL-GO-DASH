//! Module Search / Scraping multi-provider.
//!
//! Le tool `web_search` de l'agent utilise `run_search` qui route vers
//! le premier provider configuré dans l'ordre de préférence :
//! Brave → Exa → Firecrawl → SearXNG (fallback local sans clé).

pub mod brave;
pub mod catalog;
pub mod common;
pub mod exa;
pub mod firecrawl;

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_keys;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchProvider {
    Brave,
    Exa,
    Firecrawl,
}

impl SearchProvider {
    fn id(self) -> &'static str {
        match self {
            Self::Brave => "brave",
            Self::Exa => "exa",
            Self::Firecrawl => "firecrawl",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Brave => "Brave",
            Self::Exa => "Exa",
            Self::Firecrawl => "Firecrawl",
        }
    }
}

const PROVIDER_ORDER: [SearchProvider; 3] = [
    SearchProvider::Brave,
    SearchProvider::Exa,
    SearchProvider::Firecrawl,
];

/// Orchestrateur de recherche web — essaie chaque provider dans l'ordre.
pub async fn run_search(query: &str) -> Result<Vec<SearchResult>, String> {
    let query = common::validate_query(query)?;
    let mut failures = Vec::new();
    let mut configured = false;

    for provider in PROVIDER_ORDER {
        if !api_keys::has_key(provider.id()) {
            continue;
        }
        configured = true;
        match search_with_provider(provider, &query).await {
            Ok(results) if !results.is_empty() => return Ok(results),
            Ok(_) => failures.push(format!("{}: résultat vide", provider.label())),
            Err(e) => failures.push(common::sanitize_error(&e)),
        }
    }

    match crate::services::searxng::search(&query).await {
        Ok(results) if !results.is_empty() => return Ok(results),
        Ok(_) => failures.push("SearXNG: résultat vide".to_string()),
        Err(e) => failures.push(common::sanitize_error(&e)),
    }

    if configured {
        Err(format_failures(&failures))
    } else {
        Err(format!(
            "Aucun provider configuré. Fallback SearXNG indisponible: {}",
            format_failures(&failures)
        ))
    }
}

async fn search_with_provider(
    provider: SearchProvider,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    match provider {
        SearchProvider::Brave => brave::search(query).await,
        SearchProvider::Exa => exa::search(query).await,
        SearchProvider::Firecrawl => firecrawl::search(query).await,
    }
}

fn format_failures(failures: &[String]) -> String {
    if failures.is_empty() {
        return "Recherche web indisponible".to_string();
    }
    format!("Recherche web indisponible: {}", failures.join("; "))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_order_keeps_brave_first_and_searxng_external() {
        assert_eq!(
            PROVIDER_ORDER,
            [
                SearchProvider::Brave,
                SearchProvider::Exa,
                SearchProvider::Firecrawl
            ]
        );
    }

    #[test]
    fn failure_message_keeps_causes() {
        let msg = format_failures(&[
            "Brave: HTTP 429".to_string(),
            "SearXNG: timeout au démarrage".to_string(),
        ]);
        assert!(msg.contains("Brave: HTTP 429"));
        assert!(msg.contains("SearXNG: timeout au démarrage"));
    }
}
