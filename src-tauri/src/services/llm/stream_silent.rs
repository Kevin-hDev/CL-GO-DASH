//! Collecte silencieuse de réponse LLM API.
//!
//! Consomme le stream SSE sans émettre de tokens au frontend.
//! Utilisé exclusivement pour la compression de contexte.

use super::stream_http::{post_chat_request, RequestConfig};
use super::stream_tools::ToolCallAccumulator;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::stream_utils::clean_think_tags;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

/// Collecte la réponse du LLM API sans émettre de tokens au frontend.
pub async fn collect_chat_silent(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let cfg = RequestConfig { provider_id, model, messages, tools: &[], think: false };
    let resp = post_chat_request(&cfg).await.map_err(|e| e.to_string())?;
    consume_silent(resp, cancel).await
}

async fn consume_silent(
    resp: reqwest::Response,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let mut stream = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut acc = ToolCallAccumulator::new();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(std::time::Duration::from_secs(120)) => {
                return Err("Timeout compression : aucune réponse depuis 120s".to_string());
            }
            event = stream.next() => {
                let Some(event) = event else { break; };
                let event = event.map_err(|e| format!("SSE: {e}"))?;
                if event.data.trim() == "[DONE]" { continue; }
                process_chunk_silent(&event.data, &mut result, &mut acc);
            }
        }
    }

    let (tool_calls, ids) = acc.finalize();
    for (i, (name, args)) in tool_calls.iter().enumerate() {
        result.tool_calls.push((name.clone(), args.clone()));
        if let Some(id) = ids.get(i) {
            result.tool_call_ids.push(id.clone());
        }
    }

    Ok(result)
}

fn process_chunk_silent(
    data: &str,
    result: &mut StreamResult,
    acc: &mut ToolCallAccumulator,
) {
    let chunk: serde_json::Value = match serde_json::from_str(data) {
        Ok(v) => v,
        Err(_) => return,
    };
    if let Some(choice) = chunk["choices"].as_array().and_then(|a| a.first()) {
        let delta = &choice["delta"];
        if let Some(content) = delta["content"].as_str() {
            if !content.is_empty() {
                let cleaned = clean_think_tags(content);
                if !cleaned.is_empty() {
                    result.content.push_str(&cleaned);
                }
            }
        }
        if let Some(tcs) = delta["tool_calls"].as_array() {
            acc.ingest(tcs);
        }
    }
    if let Some(usage) = chunk["usage"].as_object() {
        result.eval_count = usage.get("completion_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        result.prompt_tokens = usage.get("prompt_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    }
}
