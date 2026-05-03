use zeroize::Zeroizing;

use super::types::{OAuthTokens, TokenResponse};
use crate::services::api_keys;

pub fn store_tokens(connector_id: &str, tokens: &OAuthTokens) -> Result<(), String> {
    let json = tokens.to_json()?;
    api_keys::set_mcp_token(connector_id, json.as_str())
}

pub fn get_tokens(connector_id: &str) -> Result<OAuthTokens, String> {
    let json = api_keys::get_mcp_token(connector_id)?;
    OAuthTokens::from_json(json.as_str())
}

pub fn delete_tokens(connector_id: &str) -> Result<(), String> {
    api_keys::delete_mcp_token(connector_id)
}

pub fn has_tokens(connector_id: &str) -> bool {
    api_keys::has_mcp_token(connector_id)
}

pub async fn get_valid_token(connector_id: &str) -> Result<Zeroizing<String>, String> {
    let tokens = get_tokens(connector_id)?;
    let result = tokens.access_token.clone();
    if let Some(exp) = tokens.expires_at {
        let now = chrono::Utc::now().timestamp();
        if now < exp - 30 {
            return Ok(result);
        }
        let refresh = tokens
            .refresh_token
            .as_ref()
            .ok_or("token expiré et pas de refresh_token")?;
        return refresh_access_token(connector_id, &tokens, refresh.as_str()).await;
    }
    Ok(result)
}

fn validate_https(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("endpoint non HTTPS refusé".to_string());
    }
    Ok(())
}

async fn refresh_access_token(
    connector_id: &str,
    old: &OAuthTokens,
    refresh_token: &str,
) -> Result<Zeroizing<String>, String> {
    validate_https(&old.token_endpoint)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| "erreur interne".to_string())?;

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", old.client_id.as_str()),
    ];

    let resp = client
        .post(&old.token_endpoint)
        .form(&params)
        .send()
        .await
        .map_err(|_| "échec du rafraîchissement du token".to_string())?;

    if !resp.status().is_success() {
        return Err("échec du rafraîchissement du token".to_string());
    }

    let mut raw: TokenResponse = resp
        .json()
        .await
        .map_err(|_| "réponse invalide".to_string())?;

    if raw.access_token.is_empty() {
        return Err("token manquant dans la réponse".to_string());
    }

    if raw.refresh_token.is_none() {
        raw.refresh_token = Some(refresh_token.to_string());
    }

    let new_tokens = OAuthTokens::from_response(&mut raw, &old.token_endpoint, &old.client_id);
    let result = new_tokens.access_token.clone();
    store_tokens(connector_id, &new_tokens)?;
    Ok(result)
}
