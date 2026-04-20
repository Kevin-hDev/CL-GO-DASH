//! Module LLM multi-provider — abstraction unifiée OpenAI-compat.
//!
//! Tous les providers retenus (Groq, Gemini, Mistral, Cerebras, OpenRouter, OpenAI, DeepSeek)
//! exposent une API OpenAI-compatible. Un seul client (`openai_compat.rs`) les couvre tous
//! en changeant `base_url` et `api_key`.

pub mod agent_loop;
pub mod catalog;
pub mod model_registry;
pub mod openai_compat;
mod openai_compat_parsing;
pub mod stream;
pub mod stream_convert;
mod stream_http;
mod stream_tools;
pub mod quota;
pub mod tool_capable;
pub mod types;

/// Helper non-streaming pour appels simples (utilisé par le scheduler heartbeat).
/// Retourne (contenu, tokens_totaux).
pub async fn collect_chat(
    provider_id: &str,
    model: &str,
    prompt: &str,
) -> Result<(String, u32), String> {
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
