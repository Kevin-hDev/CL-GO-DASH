use reqwest::Client;
use std::time::Duration;
use zeroize::Zeroizing;

use super::jwt;
use super::store::CodexTokens;

const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const REDIRECT_URI: &str = "http://localhost:1455/auth/callback";

pub async fn exchange_code(
    code: &str,
    code_verifier: &str,
) -> Result<CodexTokens, String> {
    let body = format!(
        "grant_type=authorization_code&client_id={CLIENT_ID}&code={}&code_verifier={}&redirect_uri={}",
        urlencoding::encode(code),
        urlencoding::encode(code_verifier),
        urlencoding::encode(REDIRECT_URI),
    );
    let resp = post_form(&body).await?;
    parse_response(resp).await
}

pub async fn refresh(refresh_val: &str) -> Result<CodexTokens, String> {
    let body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={CLIENT_ID}",
        urlencoding::encode(refresh_val),
    );
    let resp = post_form(&body).await?;
    parse_response(resp).await
}

async fn post_form(body: &str) -> Result<reqwest::Response, String> {
    let client = build_client()?;
    let resp = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("OAuth request: {e}"))?;
    check_status(&resp)?;
    Ok(resp)
}

pub async fn ensure_valid() -> Result<CodexTokens, String> {
    let creds = super::store::load()?.ok_or("non connecté à Codex")?;
    if !creds.is_expired() {
        return Ok(creds);
    }
    eprintln!("[codex] session expirée, renouvellement...");
    let new_creds = refresh(&creds.refresh).await?;
    super::store::save(&new_creds)?;
    Ok(new_creds)
}

fn build_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("http client: {e}"))
}

fn check_status(resp: &reqwest::Response) -> Result<(), String> {
    if resp.status().is_success() {
        return Ok(());
    }
    Err(format!("OAuth endpoint: HTTP {}", resp.status()))
}

async fn parse_response(resp: reqwest::Response) -> Result<CodexTokens, String> {
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("parse response: {e}"))?;

    let access = json["access_token"]
        .as_str()
        .ok_or("access_token manquant")?
        .to_string();
    let refresh_val = json["refresh_token"]
        .as_str()
        .ok_or("refresh_token manquant")?
        .to_string();
    let expires_in = json["expires_in"].as_i64().unwrap_or(3600);
    let expires_at = chrono::Utc::now().timestamp() + expires_in;

    let claims = jwt::extract_claims(&access)?;

    Ok(CodexTokens {
        access: Zeroizing::new(access),
        refresh: Zeroizing::new(refresh_val),
        expires_at,
        account_id: claims.account_id,
    })
}
