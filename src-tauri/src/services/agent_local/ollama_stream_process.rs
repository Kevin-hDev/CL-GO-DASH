use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamResult};
use crate::services::stream_utils::{clean_think_tags, compute_tps};
use tokio::sync::mpsc;

/// Traite un chunk JSON du stream Ollama.
/// Si `tool_tx` est fourni, les tool calls sont aussi envoyés via le canal pour eager dispatch.
pub fn process_chunk(
    text: &str,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
    result: &mut StreamResult,
    should_emit_done: bool,
    tool_tx: Option<&mpsc::UnboundedSender<(usize, String, serde_json::Value)>>,
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
            let idx = result.tool_calls.len();
            result.tool_calls.push((name.clone(), args.clone()));
            let _ = on_event.send(StreamEvent::ToolCall {
                name: name.clone(),
                arguments: args.clone(),
            });
            // Eager dispatch : notifier le canal si disponible
            if let Some(tx) = tool_tx {
                let _ = tx.send((idx, name, args));
            }
        }
    }

    Ok(())
}

pub fn emit_done(on_event: &AgentEventEmitter, chunk: &serde_json::Value) -> Result<(), String> {
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
