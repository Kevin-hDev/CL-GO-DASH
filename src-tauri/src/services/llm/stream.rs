//! Streaming chat SSE pour providers OpenAI-compat (avec support tools).
//!
//! Émet des `StreamEvent` compatibles avec le pattern Ollama : le frontend consomme
//! le stream de la même manière quel que soit le provider.

use super::stream_http::post_chat_request;
use super::stream_tools::ToolCallAccumulator;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent, StreamResult};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

/// Variante sans `Done` final — utilisée par l'agent loop qui émet son propre `Done`
/// après avoir agrégé tous les tours.
pub async fn stream_chat_no_done(
    on_event: &AgentEventEmitter,
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let resp = post_chat_request(provider_id, model, messages, tools).await?;
    let (result, _, _) = consume_stream(on_event, resp, cancel).await?;
    Ok(result)
}

async fn consume_stream(
    on_event: &AgentEventEmitter,
    resp: reqwest::Response,
    cancel: CancellationToken,
) -> Result<(StreamResult, u32, std::time::Instant), String> {
    let mut stream = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;
    let mut acc = ToolCallAccumulator::new();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                return Err("Timeout : aucune réponse du modèle depuis 60s".to_string());
            }
            event = stream.next() => {
                let Some(event) = event else { break; };
                let event = event.map_err(|e| format!("SSE: {e}"))?;
                if event.data.trim() == "[DONE]" { continue; }
                process_chunk(&event.data, on_event, &mut token_count, &mut first_token, &mut result, &mut acc);
            }
        }
    }

    // Finalise tool_calls accumulés et émet un event par tool.
    let (tool_calls, ids) = acc.finalize();
    for (i, (name, args)) in tool_calls.iter().enumerate() {
        let _ = on_event.send(StreamEvent::ToolCall {
            name: name.clone(),
            arguments: args.clone(),
        });
        result.tool_calls.push((name.clone(), args.clone()));
        if let Some(id) = ids.get(i) {
            result.tool_call_ids.push(id.clone());
        }
    }

    let first = first_token.unwrap_or_else(std::time::Instant::now);
    Ok((result, token_count, first))
}

fn process_chunk(
    data: &str,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
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
                result.content.push_str(content);
                *token_count += 1;
                if first_token.is_none() {
                    *first_token = Some(std::time::Instant::now());
                }
                let tps = compute_tps(*token_count, *first_token);
                let _ = on_event.send(StreamEvent::Token {
                    content: content.to_string(),
                    token_count: *token_count,
                    tps,
                });
            }
        }
        if let Some(tcs) = delta["tool_calls"].as_array() {
            acc.ingest(tcs);
        }
    }
    if let Some(usage) = chunk["usage"].as_object() {
        result.eval_count = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        result.prompt_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    }
}

fn compute_tps(count: u32, first: Option<std::time::Instant>) -> f64 {
    match first {
        Some(t) => {
            let elapsed = t.elapsed().as_secs_f64();
            if elapsed > 0.1 && count > 1 { (count - 1) as f64 / elapsed } else { 0.0 }
        }
        None => 0.0,
    }
}
