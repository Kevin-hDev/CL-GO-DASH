//! Helper HTTP partagé pour les appels de streaming OpenAI-compat.

use super::stream_convert::messages_to_openai;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::api_key_cache;
use crate::services::llm::catalog;
use std::time::Duration;

const REQUEST_TIMEOUT_SECS: u64 = 120;

/// Envoie la requête POST `/chat/completions` et renvoie la `Response` streaming.
pub async fn post_chat_request(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
) -> Result<reqwest::Response, String> {
    let spec = catalog::find(provider_id)
        .ok_or_else(|| format!("provider inconnu : {}", provider_id))?;
    let key = api_key_cache::get_key(provider_id)
        .map_err(|_| format!("clé API non configurée pour {}", spec.display_name))?;
    let url = format!("{}/chat/completions", spec.base_url);
    let openai_messages = messages_to_openai(messages);

    let mut payload = serde_json::json!({
        "model": model,
        "messages": openai_messages,
        "stream": true,
        "stream_options": { "include_usage": true },
    });
    if let Some(max) = spec.default_max_tokens {
        payload["max_tokens"] = max.into();
    }
    if think {
        match provider_id {
            "deepseek" => { payload["reasoning_effort"] = "high".into(); }
            "openai" => { payload["reasoning"] = serde_json::json!({"effort": "high"}); }
            "google" => { payload["generationConfig"] = serde_json::json!({"thinkingConfig": {"thinkingBudget": 0}}); }
            _ => {}
        }
    }
    if !tools.is_empty() {
        payload["tools"] = serde_json::Value::Array(tools.to_vec());
        payload["tool_choice"] = "auto".into();
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;

    let resp = client
        .post(&url)
        .bearer_auth(&*key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Connexion {} échouée: {e}", spec.display_name))?;

    // Capture des headers rate-limit Groq pour le quota.
    if provider_id == "groq" {
        super::quota::update_groq_limits(resp.headers());
    }

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        eprintln!("[llm stream] HTTP {} — {}", status, body);
        return Err(match status.as_u16() {
            401 | 403 => "Clé API invalide ou non autorisée".to_string(),
            400 if body.contains("must be a string") || body.contains("image") => {
                "Ce modèle ne supporte pas les images. Envoie du texte uniquement.".to_string()
            }
            404 if body.contains("tool use") => {
                "Ce modèle ne supporte pas les outils (tools). Essaie un autre modèle.".to_string()
            }
            404 if body.contains("image") => {
                "Ce modèle ne supporte pas les images. Essaie un modèle avec vision.".to_string()
            }
            413 => "Requête trop volumineuse pour ce modèle (limite TPM dépassée)".to_string(),
            429 => "Rate limit atteint, réessaie plus tard".to_string(),
            _ => format!("{} HTTP {}", spec.display_name, status),
        });
    }
    Ok(resp)
}
