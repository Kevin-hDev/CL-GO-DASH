//! Helpers de parsing/construction pour `openai_compat.rs`.

use super::types::{ChatRequest, ChatResponse, LlmError, ModelInfo, TokenUsage, ToolCall};
use reqwest::Response;

/// Construit le payload JSON pour `POST /chat/completions`.
pub fn build_payload(req: &ChatRequest, stream: bool) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "model": req.model,
        "messages": req.messages,
        "stream": stream,
    });
    if let Some(max) = req.max_tokens {
        payload["max_tokens"] = max.into();
    }
    if let Some(t) = req.temperature {
        payload["temperature"] = t.into();
    }
    if !req.tools.is_empty() {
        payload["tools"] = serde_json::to_value(&req.tools).unwrap_or(serde_json::Value::Null);
        payload["tool_choice"] = "auto".into();
    }
    payload
}

/// Parse la réponse de `GET /models`.
pub fn parse_models_list(
    body: &serde_json::Value,
    provider_id: &str,
) -> Result<Vec<ModelInfo>, LlmError> {
    let data = body["data"].as_array().ok_or_else(|| {
        LlmError::Parse(format!("champ 'data' absent ou invalide ({})", provider_id))
    })?;

    let models = data
        .iter()
        .filter_map(|m| {
            let id = m["id"].as_str()?.to_string();
            let owned_by = m["owned_by"].as_str().map(|s| s.to_string());
            let context_length = m["context_length"]
                .as_u64()
                .or_else(|| m["context_window"].as_u64())
                .or_else(|| m["max_context_length"].as_u64())
                .map(|v| v as u32);
            // OpenRouter: `supported_parameters` incluant "tools"
            // Mistral: `capabilities.function_calling: true`
            let supports_tools = m["supported_parameters"]
                .as_array()
                .map(|arr| arr.iter().any(|v| v.as_str() == Some("tools")))
                .unwrap_or(false)
                || m["capabilities"]["function_calling"].as_bool().unwrap_or(false);
            let is_chat = m["capabilities"]["completion_chat"].as_bool().unwrap_or(true);
            if !is_chat && m["capabilities"].is_object() {
                return None;
            }
            let supports_vision = m["capabilities"]["vision"].as_bool().unwrap_or(false)
                || m["architecture"]["modality"]
                    .as_str()
                    .map(|s| s.contains("image->") || s.contains("image+"))
                    .unwrap_or(false)
                || m["architecture"]["input_modalities"]
                    .as_array()
                    .map(|arr| arr.iter().any(|v| v.as_str() == Some("image")))
                    .unwrap_or(false);
            let is_free = is_price_free(&m["pricing"]["prompt"])
                && is_price_free(&m["pricing"]["completion"]);
            Some(ModelInfo {
                id,
                owned_by,
                context_length,
                supports_tools,
                supports_vision,
                supports_thinking: false,
                is_free,
            })
        })
        .collect();

    Ok(models)
}

/// Parse la réponse de `POST /chat/completions` (non-streaming).
pub fn parse_chat_response(body: &serde_json::Value) -> Result<ChatResponse, LlmError> {
    let choice = &body["choices"][0];
    let msg = &choice["message"];
    let content = msg["content"].as_str().unwrap_or("").to_string();

    let tool_calls: Vec<ToolCall> = msg["tool_calls"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|tc| serde_json::from_value(tc.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    let finish_reason = choice["finish_reason"]
        .as_str()
        .unwrap_or("stop")
        .to_string();

    let usage = TokenUsage {
        prompt_tokens: body["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
        completion_tokens: body["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
        total_tokens: body["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
    };

    Ok(ChatResponse {
        content,
        tool_calls,
        usage,
        finish_reason,
    })
}

/// Prix = "0" ou absent → gratuit.
fn is_price_free(v: &serde_json::Value) -> bool {
    match v.as_str() {
        Some(s) => s == "0" || s == "0.0" || s == "0.00",
        None => v.is_null(),
    }
}

/// Mappe un statut HTTP d'erreur vers un `LlmError` approprié.
/// On ne log jamais le body brut côté UI — uniquement en stderr pour dev.
pub async fn map_error_status(resp: Response) -> LlmError {
    let status = resp.status().as_u16();
    match status {
        401 | 403 => LlmError::Unauthorized,
        429 => {
            let retry_after_secs = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            LlmError::RateLimit { retry_after_secs }
        }
        _ => {
            let body = resp.text().await.unwrap_or_default();
            eprintln!("[llm] HTTP {} — {}", status, body);
            LlmError::Http {
                status,
                message: "erreur serveur provider".into(),
            }
        }
    }
}
