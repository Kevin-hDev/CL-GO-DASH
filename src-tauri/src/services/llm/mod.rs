//! Module LLM multi-provider — abstraction unifiée OpenAI-compat.
//!
//! Tous les providers retenus (Groq, Gemini, Mistral, Cerebras, OpenRouter, OpenAI, DeepSeek)
//! exposent une API OpenAI-compatible. Un seul client (`openai_compat.rs`) les couvre tous
//! en changeant `base_url` et `api_key`.

pub mod agent_loop;
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
            let start = cleaned[pos..]
                .find(':')
                .or_else(|| cleaned[pos..].find('='))
                .map(|i| pos + i + 1);
            if let Some(s) = start {
                let end = cleaned[s..]
                    .find(&['"', ',', '}', '&', ' '][..])
                    .map(|i| s + i)
                    .unwrap_or(cleaned.len());
                cleaned.replace_range(s..end, "[REDACTED]");
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
        return Ok((result.content, result.prompt_tokens + result.eval_count));
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
