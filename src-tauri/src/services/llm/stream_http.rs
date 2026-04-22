use super::stream_convert::messages_to_openai;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::api_key_cache;
use crate::services::llm::catalog;
use std::time::Duration;

const REQUEST_TIMEOUT_SECS: u64 = 120;

pub struct RequestConfig<'a> {
    pub provider_id: &'a str,
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
    pub tools: &'a [serde_json::Value],
    pub think: bool,
}

#[derive(Debug)]
pub enum RequestError {
    Fatal(String),
    RetryWithoutTools(String),
    RetryWithoutImages(String),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fatal(s) | Self::RetryWithoutTools(s) | Self::RetryWithoutImages(s) => {
                f.write_str(s)
            }
        }
    }
}

pub async fn post_chat_request(cfg: &RequestConfig<'_>) -> Result<reqwest::Response, RequestError> {
    let spec = catalog::find(cfg.provider_id)
        .ok_or_else(|| RequestError::Fatal(format!("provider inconnu : {}", cfg.provider_id)))?;
    let key = api_key_cache::get_key(cfg.provider_id)
        .map_err(|_| RequestError::Fatal(format!("clé API non configurée pour {}", spec.display_name)))?;
    let url = format!("{}/chat/completions", spec.base_url);
    let openai_messages = messages_to_openai(cfg.messages, cfg.provider_id);

    let mut payload = serde_json::json!({
        "model": cfg.model,
        "messages": openai_messages,
        "stream": true,
        "stream_options": { "include_usage": true },
    });
    if let Some(max) = spec.default_max_tokens {
        payload["max_tokens"] = max.into();
    }
    if cfg.think {
        match cfg.provider_id {
            "deepseek" | "google" | "openrouter" => {
                payload["reasoning_effort"] = "high".into();
            }
            "openai" => {
                payload["reasoning"] = serde_json::json!({"effort": "high"});
            }
            _ => {}
        }
    }
    if !cfg.tools.is_empty() {
        payload["tools"] = serde_json::Value::Array(cfg.tools.to_vec());
        payload["tool_choice"] = "auto".into();
    }
    if cfg.provider_id == "openrouter" {
        payload["provider"] = serde_json::json!({
            "require_parameters": true,
            "allow_fallbacks": true,
        });
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| RequestError::Fatal(format!("HTTP client: {e}")))?;

    let resp = client
        .post(&url)
        .bearer_auth(&*key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| RequestError::Fatal(format!("Connexion {} échouée: {e}", spec.display_name)))?;

    if cfg.provider_id == "groq" {
        super::quota::update_groq_limits(resp.headers());
    }

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        eprintln!("[llm stream] HTTP {} — {}", status, body);
        return Err(classify_error(status.as_u16(), &body, spec.display_name));
    }
    Ok(resp)
}

fn classify_error(status: u16, body: &str, provider_name: &str) -> RequestError {
    match status {
        401 | 403 => RequestError::Fatal("Clé API invalide ou non autorisée".into()),
        413 => RequestError::Fatal("Requête trop volumineuse (limite TPM dépassée)".into()),
        429 => RequestError::Fatal("Rate limit atteint, réessaie plus tard".into()),
        400 if body.contains("Developer instruction") || body.contains("system_instruction") => {
            RequestError::Fatal("Ce modèle ne supporte pas les instructions système via ce provider. Essaie un autre modèle.".into())
        }
        400 if body.contains("must be a string") => {
            RequestError::RetryWithoutImages("Format image non supporté par ce provider".into())
        }
        404 if body.contains("tool use") || body.contains("tool_use") => {
            RequestError::RetryWithoutTools("Aucun endpoint ne supporte les tools pour ce modèle".into())
        }
        404 if body.contains("image") => {
            RequestError::RetryWithoutImages("Aucun endpoint ne supporte les images pour ce modèle".into())
        }
        400 if body.contains("image") => {
            RequestError::RetryWithoutImages("Ce modèle ne supporte pas les images".into())
        }
        _ => RequestError::Fatal(format!("{provider_name} HTTP {status}")),
    }
}
