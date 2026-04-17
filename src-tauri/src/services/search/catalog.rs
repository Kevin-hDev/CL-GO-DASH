//! Catalogue statique des providers Search / Scraping.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SearchProviderSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str, // "search" | "scraping"
    pub signup_url: &'static str,
    pub free_tier_label: &'static str,
    pub short_description: &'static str,
    pub short_description_en: &'static str,
}

pub fn find(provider_id: &str) -> Option<&'static SearchProviderSpec> {
    SEARCH_PROVIDERS.iter().find(|p| p.id == provider_id)
}

pub const SEARCH_PROVIDERS: &[SearchProviderSpec] = &[
    SearchProviderSpec {
        id: "brave",
        display_name: "Brave Search",
        category: "search",
        signup_url: "https://api-dashboard.search.brave.com/app/keys",
        free_tier_label: "2 000 req/month",
        short_description: "Index propre, pas de dépendance Google/Bing.",
        short_description_en: "Independent index, no Google/Bing dependency.",
    },
    SearchProviderSpec {
        id: "exa",
        display_name: "Exa",
        category: "search",
        signup_url: "https://dashboard.exa.ai/api-keys",
        free_tier_label: "1 000 req/month",
        short_description: "Neural search — recherche par similarité sémantique.",
        short_description_en: "Neural search — semantic similarity search.",
    },
    SearchProviderSpec {
        id: "firecrawl",
        display_name: "Firecrawl",
        category: "scraping",
        signup_url: "https://www.firecrawl.dev/app/api-keys",
        free_tier_label: "500 crédits",
        short_description: "Extraction Markdown LLM-ready d'un URL.",
        short_description_en: "LLM-ready Markdown extraction from any URL.",
    },
    SearchProviderSpec {
        id: "serpapi",
        display_name: "SerpAPI",
        category: "search",
        signup_url: "https://serpapi.com/manage-api-key",
        free_tier_label: "100 req/month",
        short_description: "Google / Bing / DuckDuckGo SERP structuré.",
        short_description_en: "Google / Bing / DuckDuckGo structured SERP.",
    },
    SearchProviderSpec {
        id: "google_cse",
        display_name: "Google Custom Search",
        category: "search",
        signup_url: "https://developers.google.com/custom-search/v1/introduction",
        free_tier_label: "100 req/day",
        short_description: "Search Google sur domaines choisis.",
        short_description_en: "Google search on custom domains.",
    },
];
