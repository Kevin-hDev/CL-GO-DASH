use crate::services::agent_local::types_ollama::{RegistryModelDetails, RegistryTag};
use regex::Regex;
use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

const REGISTRY_URL: &str = "https://ollama.com";
const UA: &str = "CL-GO-DASH/1.0";

static META_DESC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<meta name="description" content="([^"]+)""#).unwrap()
});
static CAPA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"text-indigo-600[^>]*>([a-z]+)</span>"#).unwrap()
});
static SIZE_PILL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"x-test-size[^>]*>([^<]+)</span>"#).unwrap()
});
static CONTEXT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(\d+)K context"#).unwrap()
});
static README_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<h2[^>]*>Readme</h2>.*?<div class="prose[^"]*"[^>]*>(.*?)</div>\s*</section>"#).unwrap()
});
static TAG_ROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)href="/library/[^:"]+:([^"]+)"[^>]*md:hidden.*?([a-f0-9]{12})</span>\s*•\s*([\d.]+)\s*(GB|MB)\s*•\s*(\d+)K\s*context"#,
    )
    .unwrap()
});

pub async fn fetch_model_details(name: &str) -> Result<RegistryModelDetails, String> {
    let html = fetch_page(&format!("{REGISTRY_URL}/library/{name}")).await?;
    Ok(parse_details_html(name, &html))
}

pub async fn fetch_model_tags(name: &str) -> Result<Vec<RegistryTag>, String> {
    let html = fetch_page(&format!("{REGISTRY_URL}/library/{name}/tags")).await?;
    Ok(parse_tags_html(&html))
}

async fn fetch_page(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(url)
        .header("User-Agent", UA)
        .send()
        .await
        .map_err(|e| format!("Registre injoignable : {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Erreur registre : {}", resp.status()));
    }
    resp.text().await.map_err(|e| e.to_string())
}

fn parse_details_html(name: &str, html: &str) -> RegistryModelDetails {
    let description_short = META_DESC
        .captures(html)
        .and_then(|c| c.get(1).map(|m| html_decode(m.as_str())))
        .unwrap_or_default();

    let mut capabilities: Vec<String> = CAPA
        .captures_iter(html)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect();
    capabilities.sort();
    capabilities.dedup();

    let mut sizes: Vec<String> = SIZE_PILL
        .captures_iter(html)
        .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
        .collect();
    sizes.sort();
    sizes.dedup();

    let context_length = CONTEXT_RE
        .captures(html)
        .and_then(|c| c.get(1).and_then(|m| m.as_str().parse::<u64>().ok()))
        .map(|k| k * 1024);

    let description_long_markdown = README_RE
        .captures(html)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .map(|raw_html| htmd::convert(&raw_html).unwrap_or(raw_html))
        .unwrap_or_default();

    RegistryModelDetails {
        name: name.to_string(),
        description_short,
        description_long_markdown,
        capabilities,
        sizes,
        context_length,
    }
}

fn parse_tags_html(html: &str) -> Vec<RegistryTag> {
    let mut tags = Vec::new();
    for cap in TAG_ROW.captures_iter(html) {
        let tag_name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let digest = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
        let size_val: Option<f64> = cap.get(3).and_then(|m| m.as_str().parse().ok());
        let unit = cap.get(4).map(|m| m.as_str()).unwrap_or("GB");
        let ctx: Option<u64> = cap.get(5).and_then(|m| m.as_str().parse().ok()).map(|k: u64| k * 1024);

        let size_gb = size_val.map(|v| if unit == "MB" { v / 1024.0 } else { v });
        tags.push(RegistryTag {
            name: tag_name,
            digest_short: digest,
            size_gb,
            context_length: ctx,
        });
    }
    tags
}

fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
