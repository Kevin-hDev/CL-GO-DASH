use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent, StreamResult};
use crate::services::stream_utils::compute_tps;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::request;

pub async fn stream_chat(
    on_event: &AgentEventEmitter,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    _think: bool,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let resp = request::post_codex_stream(model, messages, tools).await?;
    consume_sse(on_event, resp, cancel).await
}

async fn consume_sse(
    on_event: &AgentEventEmitter,
    resp: reqwest::Response,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let mut sse = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;
    let start = std::time::Instant::now();

    let mut cur_tool_id: Option<String> = None;
    let mut cur_tool_name: Option<String> = None;
    let mut cur_tool_args = String::new();

    loop {
        let event = tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                return Err("Timeout Codex : 60s sans réponse".to_string());
            }
            ev = sse.next() => match ev {
                Some(Ok(e)) => e,
                Some(Err(e)) => return Err(format!("SSE: {e}")),
                None => break,
            },
        };

        if event.data.trim() == "[DONE]" {
            break;
        }
        let parsed: serde_json::Value = match serde_json::from_str(&event.data) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let etype = parsed["type"].as_str().unwrap_or("");

        match etype {
            "response.output_text.delta" => {
                let delta = parsed["delta"].as_str().unwrap_or("");
                if delta.is_empty() {
                    continue;
                }
                if first_token.is_none() {
                    first_token = Some(std::time::Instant::now());
                }
                token_count += 1;
                result.content.push_str(delta);
                let tps = compute_tps(token_count, first_token);
                let _ = on_event.send(StreamEvent::Token {
                    content: delta.to_string(),
                    token_count,
                    tps,
                });
            }
            "response.output_item.added" => {
                if let Some(item) = parsed.get("item") {
                    if item["type"].as_str() == Some("function_call") {
                        cur_tool_id = item["call_id"].as_str().map(String::from);
                        cur_tool_name = item["name"].as_str().map(String::from);
                        cur_tool_args.clear();
                    }
                }
            }
            "response.function_call_arguments.delta" => {
                let delta = parsed["delta"].as_str().unwrap_or("");
                cur_tool_args.push_str(delta);
            }
            "response.output_item.done" => {
                if let (Some(id), Some(name)) =
                    (cur_tool_id.take(), cur_tool_name.take())
                {
                    let args_json: serde_json::Value =
                        serde_json::from_str(&cur_tool_args).unwrap_or_default();
                    let _ = on_event.send(StreamEvent::ToolCall {
                        name: name.clone(),
                        arguments: args_json.clone(),
                    });
                    result.tool_calls.push((name, args_json));
                    result.tool_call_ids.push(id);
                    cur_tool_args.clear();
                }
            }
            "response.done" | "response.completed" => {
                if let Some(usage) = parsed.pointer("/response/usage") {
                    result.prompt_tokens =
                        usage["input_tokens"].as_u64().unwrap_or(0) as u32;
                    result.eval_count =
                        usage["output_tokens"].as_u64().unwrap_or(0) as u32;
                }
                break;
            }
            "response.failed" => {
                let msg = parsed
                    .pointer("/response/error/message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("erreur inconnue Codex");
                return Err(format!("Codex: {msg}"));
            }
            _ => {}
        }
    }

    let elapsed = start.elapsed();
    let final_tps = if elapsed.as_secs_f64() > 0.1 {
        token_count as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };

    let _ = on_event.send(StreamEvent::Done {
        eval_count: result.eval_count,
        eval_duration_ns: elapsed.as_nanos() as u64,
        final_tps,
        prompt_tokens: result.prompt_tokens,
        context_tokens: result.prompt_tokens + result.eval_count,
    });

    Ok(result)
}
