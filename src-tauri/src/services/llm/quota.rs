//! Récupération des infos de quota/usage par provider.
//!
//! Supporté : DeepSeek, OpenRouter, Groq/xAI (headers), Moonshot.
//! Z.ai et les autres n'exposent pas d'endpoint → retourne None.

use crate::services::api_keys;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, QUOTA_BODY_LIMIT};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderQuota {
    pub available: bool,
    pub label: String,
}

const TIMEOUT: Duration = Duration::from_secs(10);

static RATELIMIT_CACHE: std::sync::LazyLock<Mutex<HashMap<String, HashMap<String, String>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

fn quota_client() -> Option<AuthenticatedClient> {
    AuthenticatedClient::new(TIMEOUT).ok()
}

/// Appelé par stream_http après chaque réponse pour capturer les headers rate-limit.
pub fn update_ratelimit_headers(provider_id: &str, headers: &reqwest::header::HeaderMap) {
    let mut cache = match RATELIMIT_CACHE.lock() {
        Ok(c) => c,
        Err(_) => return,
    };
    let entry = cache.entry(provider_id.to_string()).or_default();
    for key in &[
        "x-ratelimit-limit-requests",
        "x-ratelimit-remaining-requests",
        "x-ratelimit-limit-tokens",
        "x-ratelimit-remaining-tokens",
    ] {
        if let Some(v) = headers.get(*key).and_then(|v| v.to_str().ok()) {
            entry.insert(key.to_string(), v.to_string());
        }
    }
}

pub async fn fetch_quota(provider_id: &str) -> Option<ProviderQuota> {
    match provider_id {
        "deepseek" => fetch_deepseek().await,
        "openrouter" => fetch_openrouter().await,
        "groq" | "xai" => fetch_ratelimit(provider_id),
        "moonshot" => fetch_moonshot().await,
        _ => None,
    }
}

fn fetch_ratelimit(provider_id: &str) -> Option<ProviderQuota> {
    let cache = RATELIMIT_CACHE.lock().ok()?;
    let entry = cache.get(provider_id)?;
    let remaining_req = entry.get("x-ratelimit-remaining-requests")?;
    let limit_req = entry.get("x-ratelimit-limit-requests")?;
    let remaining_tok = entry
        .get("x-ratelimit-remaining-tokens")
        .cloned()
        .unwrap_or_default();
    let limit_tok = entry
        .get("x-ratelimit-limit-tokens")
        .cloned()
        .unwrap_or_default();
    let label = if !remaining_tok.is_empty() {
        format!(
            "{}/{} req · {}/{} tokens",
            remaining_req, limit_req, remaining_tok, limit_tok
        )
    } else {
        format!("{}/{} requêtes restantes", remaining_req, limit_req)
    };
    Some(ProviderQuota {
        available: remaining_req != "0",
        label,
    })
}

async fn fetch_deepseek() -> Option<ProviderQuota> {
    let body = fetch_json("deepseek", "https://api.deepseek.com/user/balance").await?;
    let infos = body["balance_infos"].as_array()?;
    let first = infos.first()?;
    let total: f64 = first["total_balance"].as_str()?.parse().ok()?;
    let granted: f64 = first["granted_balance"]
        .as_str()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);
    let topped: f64 = first["topped_up_balance"]
        .as_str()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);
    let currency = first["currency"].as_str().unwrap_or("USD");
    let label = format!(
        "{:.2} {} (crédits: {:.2}, rechargé: {:.2})",
        total, currency, granted, topped
    );
    Some(ProviderQuota {
        available: body["is_available"].as_bool().unwrap_or(true),
        label,
    })
}

async fn fetch_moonshot() -> Option<ProviderQuota> {
    let body = fetch_json("moonshot", "https://api.moonshot.ai/v1/users/me/balance").await?;
    let data = &body["data"];
    let available: f64 = data["available_balance"].as_f64().unwrap_or(0.0);
    let voucher: f64 = data["voucher_balance"].as_f64().unwrap_or(0.0);
    let cash: f64 = data["cash_balance"].as_f64().unwrap_or(0.0);
    let label = format!(
        "¥{:.2} disponible (bon: ¥{:.2}, cash: ¥{:.2})",
        available, voucher, cash
    );
    Some(ProviderQuota {
        available: available > 0.0,
        label,
    })
}

async fn fetch_openrouter() -> Option<ProviderQuota> {
    let body = fetch_json("openrouter", "https://openrouter.ai/api/v1/credits").await?;
    let total: f64 = body["data"]["total_credits"].as_f64()?;
    let used: f64 = body["data"]["total_usage"].as_f64().unwrap_or(0.0);
    let remaining = total - used;
    let label = format!(
        "${:.2} restant (${:.2} utilisé / ${:.2})",
        remaining, used, total
    );
    Some(ProviderQuota {
        available: remaining > 0.0,
        label,
    })
}

async fn fetch_json(provider_id: &str, url: &str) -> Option<serde_json::Value> {
    let key = api_keys::get_key(provider_id).ok()?;
    let client = quota_client()?;
    let request = client.get(url).bearer_auth(&*key);
    let response = client.send(request).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    read_json_bounded(response, QUOTA_BODY_LIMIT).await.ok()
}

#[cfg(test)]
#[path = "quota_http_tests.rs"]
mod tests;
