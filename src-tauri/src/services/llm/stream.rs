//! Streaming chat SSE pour providers OpenAI-compat.
//! Émet des `StreamEvent` compatibles avec le pattern Ollama, de sorte que le
//! frontend peut consommer le stream de la même manière quel que soit le provider.
//!
//! Phase 6 : support chat texte sans tools. Les tools LLM API arrivent en Phase 7.

use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent, StreamResult};
use crate::services::api_keys;
use crate::services::llm::catalog;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use std::time::Duration;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

const REQUEST_TIMEOUT_SECS: u64 = 120;

pub async fn stream_chat(
    on_event: &Channel<StreamEvent>,
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let spec = catalog::find(provider_id)
        .ok_or_else(|| format!("provider inconnu : {}", provider_id))?;
    let key = api_keys::get_key(provider_id)
        .map_err(|_| format!("clé API non configurée pour {}", spec.display_name))?;

    let url = format!("{}/chat/completions", spec.base_url);

    // Conversion ChatMessage (format Ollama) → payload OpenAI-compat.
    let openai_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
            })
        })
        .collect();

    let payload = serde_json::json!({
        "model": model,
        "messages": openai_messages,
        "stream": true,
        "stream_options": { "include_usage": true },
    });

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

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        eprintln!("[llm stream] HTTP {} — {}", status, body);
        return Err(match status.as_u16() {
            401 | 403 => "Clé API invalide ou non autorisée".to_string(),
            429 => "Rate limit atteint, réessaie plus tard".to_string(),
            _ => format!("{} HTTP {}", spec.display_name, status),
        });
    }

    let start = std::time::Instant::now();
    let (result, token_count, _) = consume_stream(on_event, resp, cancel).await?;

    let elapsed_ns = start.elapsed().as_nanos() as u64;
    let final_tps = if elapsed_ns > 0 && token_count > 0 {
        token_count as f64 / (elapsed_ns as f64 / 1e9)
    } else {
        0.0
    };

    let _ = on_event.send(StreamEvent::Done {
        eval_count: result.eval_count,
        eval_duration_ns: elapsed_ns,
        final_tps,
        prompt_tokens: result.prompt_tokens,
    });

    Ok(result)
}

async fn consume_stream(
    on_event: &Channel<StreamEvent>,
    resp: reqwest::Response,
    cancel: CancellationToken,
) -> Result<(StreamResult, u32, std::time::Instant), String> {
    let mut stream = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                return Err("Annulé".to_string());
            }
            event = stream.next() => {
                let Some(event) = event else { break; };
                let event = event.map_err(|e| format!("SSE: {e}"))?;
                if event.data.trim() == "[DONE]" { continue; }
                process_chunk(&event.data, on_event, &mut token_count, &mut first_token, &mut result);
            }
        }
    }

    let first = first_token.unwrap_or_else(std::time::Instant::now);
    Ok((result, token_count, first))
}

fn process_chunk(
    data: &str,
    on_event: &Channel<StreamEvent>,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
    result: &mut StreamResult,
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
    }

    // Usage envoyé à la fin si stream_options.include_usage: true
    if let Some(usage) = chunk["usage"].as_object() {
        result.eval_count = usage
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        result.prompt_tokens = usage
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
    }
}

fn compute_tps(count: u32, first: Option<std::time::Instant>) -> f64 {
    match first {
        Some(t) => {
            let elapsed = t.elapsed().as_secs_f64();
            if elapsed > 0.1 && count > 1 {
                (count - 1) as f64 / elapsed
            } else {
                0.0
            }
        }
        None => 0.0,
    }
}
