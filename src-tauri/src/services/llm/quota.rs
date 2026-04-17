//! Récupération des infos de quota/usage par provider.
//!
//! Supporté : DeepSeek (/user/balance), OpenRouter (/credits), Groq (headers).
//! Les autres providers n'exposent pas d'endpoint → retourne None.

use crate::services::api_key_cache;
use reqwest::Client;
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

// Cache des headers rate-limit Groq (mis à jour après chaque appel chat).
static GROQ_LIMITS: std::sync::LazyLock<Mutex<HashMap<String, String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Appelé par stream_http après chaque réponse Groq pour capturer les headers.
pub fn update_groq_limits(headers: &reqwest::header::HeaderMap) {
    let mut cache = match GROQ_LIMITS.lock() {
        Ok(c) => c,
        Err(_) => return,
    };
    for key in &[
        "x-ratelimit-limit-requests",
        "x-ratelimit-remaining-requests",
        "x-ratelimit-limit-tokens",
        "x-ratelimit-remaining-tokens",
    ] {
        if let Some(v) = headers.get(*key).and_then(|v| v.to_str().ok()) {
            cache.insert(key.to_string(), v.to_string());
        }
    }
}

pub async fn fetch_quota(provider_id: &str) -> Option<ProviderQuota> {
    match provider_id {
        "deepseek" => fetch_deepseek().await,
        "openrouter" => fetch_openrouter().await,
        "groq" => fetch_groq(),
        _ => None,
    }
}

fn fetch_groq() -> Option<ProviderQuota> {
    let cache = GROQ_LIMITS.lock().ok()?;
    let remaining_req = cache.get("x-ratelimit-remaining-requests")?;
    let limit_req = cache.get("x-ratelimit-limit-requests")?;
    let remaining_tok = cache.get("x-ratelimit-remaining-tokens").cloned().unwrap_or_default();
    let limit_tok = cache.get("x-ratelimit-limit-tokens").cloned().unwrap_or_default();
    let label = if !remaining_tok.is_empty() {
        format!("{}/{} req · {}/{} tokens", remaining_req, limit_req, remaining_tok, limit_tok)
    } else {
        format!("{}/{} requêtes restantes", remaining_req, limit_req)
    };
    Some(ProviderQuota { available: remaining_req != "0", label })
}

async fn fetch_deepseek() -> Option<ProviderQuota> {
    let key = api_key_cache::get_key("deepseek").ok()?;
    let client = Client::builder().timeout(TIMEOUT).build().ok()?;
    let resp = client
        .get("https://api.deepseek.com/user/balance")
        .bearer_auth(&*key)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() { return None; }
    let body: serde_json::Value = resp.json().await.ok()?;
    let infos = body["balance_infos"].as_array()?;
    let first = infos.first()?;
    let total: f64 = first["total_balance"].as_str()?.parse().ok()?;
    let granted: f64 = first["granted_balance"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let topped: f64 = first["topped_up_balance"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
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

async fn fetch_openrouter() -> Option<ProviderQuota> {
    let key = api_key_cache::get_key("openrouter").ok()?;
    let client = Client::builder().timeout(TIMEOUT).build().ok()?;
    let resp = client
        .get("https://openrouter.ai/api/v1/credits")
        .bearer_auth(&*key)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() { return None; }
    let body: serde_json::Value = resp.json().await.ok()?;
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
