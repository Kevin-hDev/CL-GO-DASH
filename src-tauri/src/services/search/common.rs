use crate::services::agent_local::sensitive_data;
use crate::services::agent_local::types_tools::SearchResult;
use futures_util::StreamExt;

pub const MAX_RESULTS: usize = 10;
pub const MAX_QUERY_CHARS: usize = 512;
pub const MAX_TITLE_CHARS: usize = 160;
pub const MAX_SNIPPET_CHARS: usize = 300;
pub const MAX_URL_CHARS: usize = 2048;
pub const MAX_JSON_BYTES: usize = 512 * 1024;

pub fn validate_query(query: &str) -> Result<String, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err("Recherche web: requête vide".to_string());
    }
    if trimmed.chars().count() > MAX_QUERY_CHARS {
        return Err(format!(
            "Recherche web: requête trop longue (max {MAX_QUERY_CHARS} caractères)"
        ));
    }
    Ok(trimmed.to_string())
}

pub async fn read_json_bounded(
    resp: reqwest::Response,
    provider: &str,
) -> Result<serde_json::Value, String> {
    let mut body = Vec::new();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("{provider} lecture: {e}"))?;
        if body.len().saturating_add(chunk.len()) > MAX_JSON_BYTES {
            return Err(format!("{provider}: réponse trop volumineuse"));
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|e| format!("{provider} parse: {e}"))
}

pub fn make_result(title: &str, url: &str, snippet: &str) -> Option<SearchResult> {
    let url = truncate(url.trim(), MAX_URL_CHARS);
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }
    let title = truncate(title.trim(), MAX_TITLE_CHARS);
    let snippet = truncate(snippet.trim(), MAX_SNIPPET_CHARS);
    if title.is_empty() && snippet.is_empty() {
        return None;
    }
    Some(SearchResult {
        title,
        url,
        snippet,
    })
}

pub fn sanitize_error(error: &str) -> String {
    truncate(&sensitive_data::redact_text(error), 240)
}

pub fn truncate(input: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (idx, c) in input.chars().enumerate() {
        if idx >= max_chars {
            out.push_str("...");
            return out;
        }
        out.push(c);
    }
    out
}
