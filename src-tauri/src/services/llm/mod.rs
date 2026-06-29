//! Module LLM multi-provider — abstraction unifiée OpenAI-compat.
//!
//! Tous les providers retenus (Groq, Gemini, Mistral, Cerebras, OpenRouter, OpenAI, DeepSeek)
//! exposent une API OpenAI-compatible. Un seul client (`openai_compat.rs`) les couvre tous
//! en changeant `base_url` et `api_key`.

pub mod agent_loop;
mod agent_loop_compression;
mod agent_loop_message;
mod agent_loop_tools;
pub mod catalog;
pub mod compress_hook;
pub mod model_registry;
pub mod openai_compat;
mod openai_compat_models;
mod openai_compat_parsing;
#[cfg(test)]
mod openai_compat_parsing_tests;
pub(crate) mod providers;
pub mod quota;
pub mod registry_search;
mod retry;
pub mod stream;
mod stream_chunk;
#[cfg(test)]
mod stream_chunk_tests;
mod stream_consume;
pub mod stream_convert;
mod stream_http;
mod stream_reasoning;
#[cfg(test)]
mod stream_reasoning_tests;
mod stream_silent;
mod stream_sse;
mod stream_tools;
mod timeouts;
pub mod tool_capable;
mod tool_schema;
pub mod types;
pub mod vision;

#[cfg(test)]
#[path = "sanitize_log_body_tests.rs"]
mod sanitize_log_body_tests;

pub(crate) fn sanitize_log_body(body: &str) -> String {
    let truncated = if body.len() > 200 {
        &body[..body
            .char_indices()
            .nth(200)
            .map(|(i, _)| i)
            .unwrap_or(body.len())]
    } else {
        body
    };
    let mut cleaned = truncated.replace(|c: char| c.is_control(), " ");
    for sensitive in &[
        "key",
        "token",
        "secret",
        "password",
        "authorization",
        "api_key",
        "apikey",
    ] {
        if let Some(pos) = cleaned.to_lowercase().find(sensitive) {
            // Localise le séparateur de valeur (':' pour JSON/YAML, '=' pour
            // query params / headers).
            let sep = cleaned[pos..]
                .find(':')
                .or_else(|| cleaned[pos..].find('='))
                .map(|i| pos + i);
            if let Some(sep_pos) = sep {
                // Cas JSON : "api_key":"value" — le séparateur ':' est suivi
                // d'un '"' ouvrant qu'il faut ignorer pour atteindre la vraie
                // valeur. Sinon (query param / header), la valeur suit le ':'.
                let bytes = cleaned.as_bytes();
                let is_json = sep_pos + 1 < bytes.len() && bytes[sep_pos + 1] == b'"';
                let value_start = if is_json { sep_pos + 2 } else { sep_pos + 1 };
                // Fin de la valeur : en contexte JSON, on cherche le '"'
                // fermant (pour capter "Bearer sk-xxx" avec espaces). Sinon,
                // on s'arrête au prochain délimiteur classique.
                let end = if is_json {
                    cleaned[value_start..]
                        .find('"')
                        .map(|i| value_start + i)
                        .unwrap_or(cleaned.len())
                } else {
                    cleaned[value_start..]
                        .find(&['"', ',', '}', '&', ' '][..])
                        .map(|i| value_start + i)
                        .unwrap_or(cleaned.len())
                };
                if value_start <= end {
                    cleaned.replace_range(value_start..end, "[REDACTED]");
                }
            }
        }
    }
    cleaned
}

/// Helper non-streaming pour appels simples (utilisé par le scheduler heartbeat).
/// Retourne (contenu, tokens_totaux).
pub async fn collect_chat(
    provider_id: &str,
    model: &str,
    prompt: &str,
) -> Result<(String, u32), String> {
    if provider_id == "codex-oauth" {
        let msg = crate::services::agent_local::types_ollama::ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        };
        let result = stream::collect_chat_silent(
            provider_id,
            model,
            &[msg],
            tokio_util::sync::CancellationToken::new(),
        )
        .await?;
        let total = crate::services::token_counting::sum_real_counts(
            result.prompt_tokens,
            result.eval_count,
        )
        .unwrap_or(0);
        return Ok((result.content, total));
    }
    let provider = openai_compat::OpenAiCompatProvider::new(provider_id).map_err(String::from)?;
    let req = types::ChatRequest {
        model: model.to_string(),
        messages: vec![types::ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
            tool_call_id: None,
            tool_calls: None,
        }],
        ..Default::default()
    };
    let resp = provider.chat_completion(req).await.map_err(String::from)?;
    let total = resp.usage.completion_tokens + resp.usage.prompt_tokens;
    Ok((resp.content, total))
}
