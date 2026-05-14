use super::catalog::{ForecastModelSpec, ForecastProviderSpec};
use super::model_details_github;
use regex::Regex;
use reqwest::Client;
use serde::Serialize;
use std::sync::LazyLock;
use std::time::Duration;

const UA: &str = "CL-GO-DASH/1.0";

static HF_RELATIVE_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\]\((\./|/)?([^)]+)\)"#).unwrap());
static HF_RELATIVE_IMAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"!\[([^\]]*)\]\((\./|/)?([^)]+)\)"#).unwrap());

#[derive(Debug, Clone, Serialize)]
pub struct ForecastModelDetails {
    pub description_short: String,
    pub description_long_markdown: String,
    pub source_url: String,
    pub source_label: String,
    pub license: Option<String>,
    pub pipeline_tag: Option<String>,
    pub library_name: Option<String>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
    pub tags: Vec<String>,
}

pub async fn fetch(
    model: &ForecastModelSpec,
    provider: Option<&ForecastProviderSpec>,
) -> Result<ForecastModelDetails, String> {
    if let Some(repo) = model.github_repo {
        return model_details_github::fetch_github(repo, model.github_revision).await;
    }
    if let Some(repo) = model.hf_repo {
        return fetch_hugging_face(repo, model.hf_revision).await;
    }
    Ok(provider_fallback(model, provider))
}

async fn fetch_hugging_face(
    repo: &str,
    revision: Option<&str>,
) -> Result<ForecastModelDetails, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| "Impossible de préparer la requête modèle".to_string())?;

    let api_url = format!("https://huggingface.co/api/models/{repo}");
    let meta: serde_json::Value = client
        .get(&api_url)
        .header("User-Agent", UA)
        .send()
        .await
        .map_err(|_| "Impossible de charger les métadonnées du modèle".to_string())?
        .json()
        .await
        .map_err(|_| "Impossible de lire les métadonnées du modèle".to_string())?;

    let rev = revision.unwrap_or("main");
    let readme_url = format!("https://huggingface.co/{repo}/raw/{rev}/README.md");
    let markdown = match client
        .get(&readme_url)
        .header("User-Agent", UA)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
        _ => String::new(),
    };

    let card = meta.get("cardData").and_then(|v| v.as_object());
    let description_short = card
        .and_then(|c| c.get("model_summary"))
        .and_then(|v| v.as_str())
        .or_else(|| card.and_then(|c| c.get("description")).and_then(|v| v.as_str()))
        .unwrap_or_default()
        .trim()
        .to_string();

    let tags = meta
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .filter(|tag| !tag.starts_with("region:"))
                .take(16)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(ForecastModelDetails {
        description_short,
        description_long_markdown: absolutize_hf_markdown(repo, rev, strip_frontmatter(markdown)),
        source_url: format!("https://huggingface.co/{repo}"),
        source_label: "Hugging Face".to_string(),
        license: card
            .and_then(|c| c.get("license"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        pipeline_tag: meta
            .get("pipeline_tag")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        library_name: meta
            .get("library_name")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        downloads: meta.get("downloads").and_then(|v| v.as_u64()),
        likes: meta.get("likes").and_then(|v| v.as_u64()),
        tags,
    })
}

fn provider_fallback(
    model: &ForecastModelSpec,
    provider: Option<&ForecastProviderSpec>,
) -> ForecastModelDetails {
    ForecastModelDetails {
        description_short: provider
            .map(|p| p.short_description.to_string())
            .unwrap_or_else(|| model.display_name.to_string()),
        description_long_markdown: String::new(),
        source_url: provider
            .map(|p| p.signup_url.to_string())
            .unwrap_or_default(),
        source_label: provider
            .map(|p| p.display_name.to_string())
            .unwrap_or_else(|| model.provider_id.to_string()),
        license: None,
        pipeline_tag: None,
        library_name: None,
        downloads: None,
        likes: None,
        tags: Vec::new(),
    }
}

fn absolutize_hf_markdown(repo: &str, revision: &str, markdown: String) -> String {
    let base_blob = format!("https://huggingface.co/{repo}/blob/{revision}/");
    let base_resolve = format!("https://huggingface.co/{repo}/resolve/{revision}/");
    let step1 = HF_RELATIVE_IMAGE.replace_all(&markdown, |caps: &regex::Captures| {
        format!("![{}]({}{})", &caps[1], base_resolve, &caps[3])
    });
    HF_RELATIVE_LINK
        .replace_all(&step1, |caps: &regex::Captures| {
            format!("]({}{})", base_blob, &caps[2])
        })
        .into_owned()
}

fn strip_frontmatter(markdown: String) -> String {
    if let Some(rest) = markdown.strip_prefix("---\n") {
        let mut parts = rest.splitn(2, "\n---\n");
        let _frontmatter = parts.next();
        if let Some(content) = parts.next() {
            return content.trim_start().to_string();
        }
    }
    markdown
}
