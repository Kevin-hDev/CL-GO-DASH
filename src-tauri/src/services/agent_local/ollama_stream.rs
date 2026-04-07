use crate::services::agent_local::types_ollama::{ChatRequest, StreamEvent, StreamResult};
use futures_util::StreamExt;
use tauri::ipc::Channel;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;
use tokio_util::sync::CancellationToken;

const BASE_URL: &str = "http://localhost:11434";

pub async fn stream_chat(
    on_event: &Channel<StreamEvent>,
    request: &ChatRequest,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{BASE_URL}/api/chat"))
        .json(request)
        .send()
        .await
        .map_err(|e| format!("Connexion Ollama impossible: {e}"))?;

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
            line = lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if let Err(e) = process_chunk(
                            &text, on_event, &mut token_count, &mut first_token, &mut result,
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

fn process_chunk(
    text: &str,
    on_event: &Channel<StreamEvent>,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
    result: &mut StreamResult,
) -> Result<(), String> {
    let chunk: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("JSON invalide: {e}"))?;

    if chunk["done"].as_bool() == Some(true) {
        result.eval_count = chunk["eval_count"].as_u64().unwrap_or(0) as u32;
        result.prompt_tokens = chunk["prompt_eval_count"].as_u64().unwrap_or(0) as u32;
        return emit_done(on_event, &chunk);
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

fn emit_done(on_event: &Channel<StreamEvent>, chunk: &serde_json::Value) -> Result<(), String> {
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

/// Nettoie les balises <think>/</think> et /think /no_think (bug Qwen3)
fn clean_think_tags(content: &str) -> String {
    content
        .replace("<think>", "")
        .replace("</think>", "")
        .replace("/think", "")
        .replace("/no_think", "")
}
