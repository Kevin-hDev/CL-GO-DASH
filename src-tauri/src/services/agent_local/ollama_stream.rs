use crate::services::agent_local::types_ollama::{ChatMessage, ChatRequest, StreamEvent, StreamResult};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::OLLAMA_BASE_URL;
use futures_util::StreamExt;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;
use tokio_util::sync::CancellationToken;
const COLLECT_TIMEOUT_SECS: u64 = 60;

/// Appel Ollama non-interactif (sans streaming UI).
/// Utilisé par le scheduler pour les réveils : le prompt est envoyé, la réponse
/// complète est accumulée, et on renvoie (contenu, tokens).
pub async fn collect_chat(model: &str, messages: Vec<ChatMessage>) -> Result<(String, u32), String> {
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": false,
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(COLLECT_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Client HTTP : {e}"))?;

    let resp = client
        .post(format!("{OLLAMA_BASE_URL}/api/chat"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Connexion Ollama impossible : {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama HTTP {}", resp.status()));
    }

    let value: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Réponse Ollama invalide : {e}"))?;

    let content = value["message"]["content"].as_str().unwrap_or("").to_string();
    let tokens = value["eval_count"].as_u64().unwrap_or(0) as u32;
    Ok((content, tokens))
}

pub async fn stream_chat(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    stream_chat_inner(on_event, request, cancel, true).await
}

pub async fn stream_chat_no_done(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    stream_chat_inner(on_event, request, cancel, false).await
}

async fn stream_chat_inner(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: CancellationToken,
    emit_done: bool,
) -> Result<StreamResult, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{OLLAMA_BASE_URL}/api/chat"))
        .json(request)
        .send()
        .await
        .map_err(|e| format!("Connexion Ollama impossible: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if let Some(retry_req) = build_retry_request(request, &body) {
            let feature = if retry_req.think != request.think { "thinking" }
                else if retry_req.tools != request.tools { "tools" }
                else { "images" };
            eprintln!("[ollama-stream] modèle sans {feature}, retry");
            return Box::pin(stream_chat_inner(on_event, &retry_req, cancel, emit_done)).await;
        }
        let msg = format!("Ollama HTTP {status}: {body}");
        eprintln!("[ollama-stream] {msg}");
        let _ = on_event.send(StreamEvent::Error { message: "Erreur serveur Ollama".into() });
        return Err(msg);
    }

    let byte_stream = resp
        .bytes_stream()
        .map(|r| r.map_err(|e| std::io::Error::other(e)));
    let mut lines = BufReader::new(StreamReader::new(byte_stream)).lines();

    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;
    let mut result = StreamResult::default();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                return Err("Annulé".to_string());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
                let msg = "Timeout : aucune réponse d'Ollama depuis 5 min".to_string();
                let _ = on_event.send(StreamEvent::Error { message: msg.clone() });
                return Err(msg);
            }
            line = lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if let Err(e) = process_chunk(
                            &text, on_event, &mut token_count, &mut first_token, &mut result, emit_done,
                        ) {
                            let _ = on_event.send(StreamEvent::Error { message: e.clone() });
                            return Err(e);
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let msg = format!("Erreur lecture stream: {e}");
                        let _ = on_event.send(StreamEvent::Error { message: msg.clone() });
                        return Err(msg);
                    }
                }
            }
        }
    }
    Ok(result)
}

fn build_retry_request(request: &ChatRequest, error_body: &str) -> Option<ChatRequest> {
    let mut retry = request.clone();
    let mut changed = false;
    if error_body.contains("does not support thinking") && request.think == Some(true) {
        retry.think = Some(false);
        changed = true;
    }
    if error_body.contains("does not support tools") && request.tools.is_some() {
        retry.tools = None;
        changed = true;
    }
    if error_body.contains("does not support images") {
        for msg in &mut retry.messages {
            msg.images = None;
        }
        changed = true;
    }
    if changed { Some(retry) } else { None }
}

fn process_chunk(
    text: &str,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
    result: &mut StreamResult,
    should_emit_done: bool,
) -> Result<(), String> {
    let chunk: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("JSON invalide: {e}"))?;

    if let Some(err) = chunk["error"].as_str() {
        eprintln!("[ollama-stream] erreur modèle: {err}");
        return Err(format!("Ollama: {err}"));
    }

    if chunk["done"].as_bool() == Some(true) {
        result.eval_count = chunk["eval_count"].as_u64().unwrap_or(0) as u32;
        result.prompt_tokens = chunk["prompt_eval_count"].as_u64().unwrap_or(0) as u32;
        if should_emit_done {
            return emit_done(on_event, &chunk);
        }
        return Ok(());
    }

    let msg = &chunk["message"];

    if let Some(thinking) = msg["thinking"].as_str() {
        if !thinking.is_empty() {
            result.thinking.push_str(thinking);
            let _ = on_event.send(StreamEvent::Thinking {
                content: thinking.to_string(),
            });
        }
    }

    if let Some(content) = msg["content"].as_str() {
        let cleaned = clean_think_tags(content);
        if !cleaned.is_empty() {
            result.content.push_str(&cleaned);
            *token_count += 1;
            if first_token.is_none() {
                *first_token = Some(std::time::Instant::now());
            }
            let tps = compute_tps(*token_count, *first_token);
            let _ = on_event.send(StreamEvent::Token {
                content: cleaned,
                token_count: *token_count,
                tps,
            });
        }
    }

    if let Some(tool_calls) = msg["tool_calls"].as_array() {
        for tc in tool_calls {
            let func = &tc["function"];
            let name = func["name"].as_str().unwrap_or("").to_string();
            let args = func["arguments"].clone();
            result.tool_calls.push((name.clone(), args.clone()));
            let _ = on_event.send(StreamEvent::ToolCall {
                name,
                arguments: args,
            });
        }
    }

    Ok(())
}

fn emit_done(on_event: &AgentEventEmitter, chunk: &serde_json::Value) -> Result<(), String> {
    let ec = chunk["eval_count"].as_u64().unwrap_or(0) as u32;
    let ed = chunk["eval_duration"].as_u64().unwrap_or(1);
    let pt = chunk["prompt_eval_count"].as_u64().unwrap_or(0) as u32;
    let final_tps = if ed > 0 { ec as f64 / (ed as f64 / 1e9) } else { 0.0 };

    let _ = on_event.send(StreamEvent::Done {
        eval_count: ec,
        eval_duration_ns: ed,
        final_tps,
        prompt_tokens: pt,
    });
    Ok(())
}

use crate::services::stream_utils::{compute_tps, clean_think_tags};
