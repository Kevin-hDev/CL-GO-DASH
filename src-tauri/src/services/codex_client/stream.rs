use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{
    ChatMessage, StreamEvent, StreamOutcome, StreamResult,
};
use crate::services::compress::realtime_budget::RealtimeBudget;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::request;

pub const CODEX_IDLE_TIMEOUT_SECS: u64 = 180;

pub use super::stream_silent::{collect_chat_silent, collect_chat_silent_for_compression};

pub async fn stream_chat_with_budget(
    on_event: &AgentEventEmitter,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, String> {
    let resp = request::post_codex_stream(model, messages, tools, think, reasoning_mode).await?;
    consume_sse(on_event, resp, cancel, buffer_content, realtime_budget).await
}

async fn consume_sse(
    on_event: &AgentEventEmitter,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    mut realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, String> {
    let mut sse = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;

    let mut cur_tool_id: Option<String> = None;
    let mut cur_tool_name: Option<String> = None;
    let mut cur_tool_args = String::new();
    let mut interrupted = false;

    loop {
        let event = tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(std::time::Duration::from_secs(CODEX_IDLE_TIMEOUT_SECS)) => {
                return Err("Timeout Codex : 180s sans réponse".to_string());
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
            "response.reasoning_summary_text.delta" => {
                let delta = parsed["delta"].as_str().unwrap_or("");
                if !delta.is_empty() {
                    result.thinking.push_str(delta);
                    let _ = on_event.send(StreamEvent::Thinking {
                        content: delta.to_string(),
                    });
                }
            }
            "response.output_text.delta" => {
                let delta = parsed["delta"].as_str().unwrap_or("");
                if delta.is_empty() {
                    continue;
                }
                crate::services::agent_local::stream_buffer::record_content(
                    on_event,
                    &mut result,
                    delta.to_string(),
                    &mut token_count,
                    &mut first_token,
                    buffer_content,
                );
                if should_interrupt(
                    &mut realtime_budget,
                    token_count,
                    cur_tool_id.is_some() || !result.tool_calls.is_empty(),
                ) {
                    interrupted = true;
                    break;
                }
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
                if let (Some(id), Some(name)) = (cur_tool_id.take(), cur_tool_name.take()) {
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
                    result.prompt_tokens = usage["input_tokens"].as_u64().map(|v| v as u32);
                    result.eval_count = usage["output_tokens"].as_u64().map(|v| v as u32);
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

    if interrupted {
        Ok(StreamOutcome::InterruptedForCompression(result))
    } else {
        Ok(StreamOutcome::Completed(result))
    }
}

fn should_interrupt(
    budget: &mut Option<RealtimeBudget>,
    token_count: u32,
    has_tool_call: bool,
) -> bool {
    !has_tool_call
        && budget
            .as_mut()
            .is_some_and(|budget| budget.should_interrupt(token_count))
}
