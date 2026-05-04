use std::time::Duration;

use reqwest::Client;

use super::types::{AuthServerMetadata, ProtectedResourceMetadata};

const TIMEOUT: Duration = Duration::from_secs(15);

pub async fn discover_auth_server(endpoint: &str) -> Result<AuthServerMetadata, String> {
    if let Some(meta) = hardcoded_override(endpoint) {
        return Ok(meta);
    }

    let client = Client::builder()
        .timeout(TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "erreur interne".to_string())?;

    let issuer = discover_issuer(&client, endpoint).await?;
    fetch_auth_server_metadata(&client, &issuer).await
}

async fn discover_issuer(client: &Client, endpoint: &str) -> Result<String, String> {
    let resp = client
        .post(endpoint)
        .header("Content-Type", "application/json")
        .body(r#"{"jsonrpc":"2.0","method":"initialize","id":1}"#)
        .send()
        .await
        .map_err(|_| "impossible de contacter le serveur MCP".to_string())?;

    let status = resp.status().as_u16();

    if status == 401 || status == 403 {
        if let Some(header) = resp.headers().get("www-authenticate").and_then(|v| v.to_str().ok()) {
            if let Some(url) = extract_resource_metadata_url(header) {
                if let Ok(issuer) = fetch_issuer_from_resource_meta(client, &url).await {
                    return Ok(issuer);
                }
            }
        }
    }

    let base_url = endpoint_base_url(endpoint)?;

    let candidates = [
        format!("{base_url}/.well-known/oauth-protected-resource"),
        format!("{base_url}/.well-known/oauth-protected-resource/mcp"),
    ];

    for url in &candidates {
        if let Ok(issuer) = fetch_issuer_from_resource_meta(client, url).await {
            return Ok(issuer);
        }
    }

    let auth_candidates = [
        format!("{base_url}/.well-known/oauth-authorization-server"),
        format!("{base_url}/.well-known/openid-configuration"),
    ];

    for url in &auth_candidates {
        let resp = client.get(url).send().await;
        if let Ok(r) = resp {
            if r.status().is_success() {
                return Ok(base_url);
            }
        }
    }

    Err("serveur d'autorisation non trouvé pour ce service".to_string())
}

fn extract_resource_metadata_url(header: &str) -> Option<String> {
    let mut found = None;
    for segment in header.split([',', ' ']) {
        let trimmed = segment.trim();
        if let Some(rest) = trimmed.strip_prefix("resource_metadata=") {
            let url = rest.trim_matches('"').trim();
            if url.starts_with("https://") {
                found = Some(url.to_string());
            }
        }
    }
    found
}

async fn fetch_issuer_from_resource_meta(
    client: &Client,
    url: &str,
) -> Result<String, String> {
    let resp = client.get(url).send().await.map_err(|_| "serveur non disponible".to_string())?;
    if !resp.status().is_success() {
        return Err("not found".to_string());
    }
    let meta: ProtectedResourceMetadata = super::bounded_json(resp).await?;
    meta.authorization_servers
        .and_then(|s| s.into_iter().next())
        .ok_or("no auth server".to_string())
}

async fn fetch_auth_server_metadata(
    client: &Client,
    issuer: &str,
) -> Result<AuthServerMetadata, String> {
    let issuer_clean = issuer.trim_end_matches('/');

    let candidates = [
        format!("{issuer_clean}/.well-known/oauth-authorization-server"),
        format!("{issuer_clean}/.well-known/openid-configuration"),
    ];

    if let Ok(parsed) = reqwest::Url::parse(issuer) {
        if !parsed.path().is_empty() && parsed.path() != "/" {
            let base = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));
            let with_path = format!(
                "{base}/.well-known/oauth-authorization-server{}",
                parsed.path().trim_end_matches('/')
            );
            if let Ok(meta) = try_fetch_metadata(client, &with_path).await {
                return Ok(meta);
            }
        }
    }

    for url in &candidates {
        if let Ok(meta) = try_fetch_metadata(client, url).await {
            return Ok(meta);
        }
    }

    Err("métadonnées du serveur d'autorisation non trouvées".to_string())
}

async fn try_fetch_metadata(
    client: &Client,
    url: &str,
) -> Result<AuthServerMetadata, String> {
    let resp = client.get(url).send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() {
        return Err("not found".to_string());
    }
    super::bounded_json(resp).await
}

fn endpoint_base_url(endpoint: &str) -> Result<String, String> {
    let parsed = reqwest::Url::parse(endpoint).map_err(|_| "URL invalide".to_string())?;
    let host = parsed.host_str().unwrap_or("");
    let port = parsed.port().map(|p| format!(":{p}")).unwrap_or_default();
    Ok(format!("{}://{host}{port}", parsed.scheme()))
}

fn hardcoded_override(endpoint: &str) -> Option<AuthServerMetadata> {
    let host = reqwest::Url::parse(endpoint).ok()?.host_str()?.to_string();

    if host == "api.githubcopilot.com" || host.ends_with(".githubcopilot.com") {
        return Some(AuthServerMetadata {
            authorization_endpoint: "https://github.com/login/oauth/authorize".to_string(),
            token_endpoint: "https://github.com/login/oauth/access_token".to_string(),
            registration_endpoint: None,
            code_challenge_methods_supported: Some(vec!["S256".to_string()]),
        });
    }

    if host.ends_with(".googleapis.com") {
        return Some(AuthServerMetadata {
            authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_endpoint: "https://oauth2.googleapis.com/token".to_string(),
            registration_endpoint: None,
            code_challenge_methods_supported: Some(vec!["S256".to_string()]),
        });
    }

    None
}
