use crate::services::agent_local::agent_settings;
use crate::services::agent_local::circuit_breaker;
use crate::services::agent_local::ollama_stream;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_executor;
use crate::services::agent_local::tool_result_budget;
use crate::services::agent_local::types_ollama::{
    ChatMessage, ChatRequest, StreamEvent,
};
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

use crate::services::agent_local::OLLAMA_BASE_URL;

const MAX_TURNS: usize = 30;

pub async fn run_agent_loop(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    model: &str,
    tools: Vec<serde_json::Value>,
    think: bool,
    working_dir: PathBuf,
    session_id: String,
    cancel: CancellationToken,
) -> Result<u32, String> {
    let mut total_eval: u32 = 0;
    let mut total_prompt: u32 = 0;
    let start = std::time::Instant::now();
    let mut breaker = circuit_breaker::CircuitBreaker::new();

    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }

        tool_result_budget::apply_budget(messages);
        let request = build_request(model, messages, &tools, think);
        let result = ollama_stream::stream_chat_no_done(on_event, &request, cancel.clone()).await?;

        total_eval += result.eval_count;
        total_prompt += result.prompt_tokens;
        messages.push(build_assistant_message(&result));

        if result.tool_calls.is_empty() {
            break;
        }

        if turn == MAX_TURNS - 1 {
            let _ = on_event.send(StreamEvent::Error {
                message: "Limite de tours atteinte".to_string(),
            });
            break;
        }

        if let Err(msg) = breaker.check(&result.tool_calls) {
            let _ = on_event.send(StreamEvent::Error { message: msg });
            break;
        }

        let mode = agent_settings::get_permission_mode().await;
        tool_executor::run_tools(
            on_event,
            messages,
            &result.tool_calls,
            &working_dir,
            &mode,
            &session_id,
            cancel.clone(),
        )
        .await;

        let _ = on_event.send(StreamEvent::TurnEnd {});
    }

    let elapsed_ns = start.elapsed().as_nanos() as u64;
    let final_tps = if elapsed_ns > 0 {
        total_eval as f64 / (elapsed_ns as f64 / 1e9)
    } else {
        0.0
    };
    let _ = on_event.send(StreamEvent::Done {
        eval_count: total_eval,
        eval_duration_ns: elapsed_ns,
        final_tps,
        prompt_tokens: total_prompt,
    });

    decharge_gpu(model).await;
    Ok(total_eval + total_prompt)
}

fn build_request(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
) -> ChatRequest {
    let keep_alive = crate::services::config::read_config()
        .map(|c| c.advanced.keep_alive)
        .unwrap_or_else(|_| "5m".to_string());
    let keep_alive = if keep_alive == "forever" { "-1m".to_string() } else { keep_alive };

    ChatRequest {
        model: model.to_string(),
        messages: messages.to_vec(),
        stream: true,
        tools: if tools.is_empty() { None } else { Some(tools.to_vec()) },
        options: None,
        keep_alive: Some(keep_alive),
        think: Some(think),
    }
}

fn build_assistant_message(
    result: &crate::services::agent_local::types_ollama::StreamResult,
) -> ChatMessage {
    let tool_calls = if result.tool_calls.is_empty() {
        None
    } else {
        Some(
            result
                .tool_calls
                .iter()
                .enumerate()
                .map(|(i, (name, args))| crate::services::agent_local::types_ollama::ToolCallOllama {
                    id: result.tool_call_ids.get(i).cloned(),
                    function: crate::services::agent_local::types_ollama::ToolCallFunction {
                        name: name.clone(),
                        arguments: args.clone(),
                    },
                })
                .collect(),
        )
    };
    ChatMessage {
        role: "assistant".to_string(),
        content: result.content.clone(),
        images: None,
        tool_calls,
        tool_name: None,
        tool_call_id: None,
    }
}

async fn decharge_gpu(model: &str) {
    let keep_alive = crate::services::config::read_config()
        .map(|c| c.advanced.keep_alive)
        .unwrap_or_else(|_| "5m".to_string());
    if keep_alive != "0" {
        return;
    }
    let client = reqwest::Client::new();
    let _ = client
        .post(format!("{OLLAMA_BASE_URL}/api/chat"))
        .json(&serde_json::json!({
            "model": model,
            "messages": [],
            "keep_alive": "0"
        }))
        .send()
        .await;
}
