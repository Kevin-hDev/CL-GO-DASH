use super::model_details::ForecastModelDetails;
use regex::Regex;
use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

const UA: &str = "CL-GO-DASH/1.0";

static GH_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\]\(([^)]+)\)"#).unwrap());
static GH_RELATIVE_IMAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"!\[([^\]]*)\]\(([^)]+)\)"#).unwrap());
static GH_HTML_SRC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"src="([^"]+)""#).unwrap());

pub async fn fetch_github(repo: &str, revision: Option<&str>) -> Result<ForecastModelDetails, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| "Impossible de préparer la requête GitHub".to_string())?;

    let api_url = format!("https://api.github.com/repos/{repo}");
    let meta: serde_json::Value = client
        .get(&api_url)
        .header("User-Agent", UA)
        .send()
        .await
        .map_err(|_| "Impossible de charger les métadonnées GitHub".to_string())?
        .json()
        .await
        .map_err(|_| "Impossible de lire les métadonnées GitHub".to_string())?;

    let rev = revision.unwrap_or("main");
    let readme_url = format!("https://raw.githubusercontent.com/{repo}/{rev}/README.md");
    let markdown = match client
        .get(&readme_url)
        .header("User-Agent", UA)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
        _ => String::new(),
    };

    Ok(ForecastModelDetails {
        description_short: meta
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        description_long_markdown: absolutize_github_markdown(repo, rev, markdown),
        source_url: format!("https://github.com/{repo}"),
        source_label: "GitHub".to_string(),
        license: meta
            .get("license")
            .and_then(|v| v.get("spdx_id"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        pipeline_tag: None,
        library_name: None,
        downloads: None,
        likes: meta.get("stargazers_count").and_then(|v| v.as_u64()),
        tags: Vec::new(),
    })
}

fn absolutize_github_markdown(repo: &str, revision: &str, markdown: String) -> String {
    let base_blob = format!("https://github.com/{repo}/blob/{revision}/");
    let base_raw = format!("https://raw.githubusercontent.com/{repo}/{revision}/");
    let step1 = GH_RELATIVE_IMAGE.replace_all(&markdown, |caps: &regex::Captures| {
        let target = &caps[2];
        if is_relative_target(target) {
            format!("![{}]({}{})", &caps[1], base_raw, normalize_relative_target(target))
        } else {
            caps[0].to_string()
        }
    });
    let step2 = GH_HTML_SRC.replace_all(&step1, |caps: &regex::Captures| {
        let target = &caps[1];
        if is_relative_target(target) {
            format!("src=\"{}{}\"", base_raw, normalize_relative_target(target))
        } else {
            caps[0].to_string()
        }
    });
    GH_LINK
        .replace_all(&step2, |caps: &regex::Captures| {
            let target = &caps[1];
            if is_relative_target(target) {
                format!("]({}{})", base_blob, normalize_relative_target(target))
            } else {
                caps[0].to_string()
            }
        })
        .into_owned()
}

fn is_relative_target(target: &str) -> bool {
    !target.is_empty()
        && !target.starts_with("http://")
        && !target.starts_with("https://")
        && !target.starts_with("data:")
        && !target.starts_with('#')
        && !target.starts_with("mailto:")
}

fn normalize_relative_target(target: &str) -> &str {
    target.strip_prefix("./").unwrap_or(target)
}
