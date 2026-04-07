use crate::services::agent_local::ollama_stream;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::types_ollama::{
    ChatMessage, ChatOptions, ChatRequest, StreamEvent,
};
use std::path::PathBuf;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

const MAX_TURNS: usize = 50;
const BASE_URL: &str = "http://localhost:11434";

pub async fn run_agent_loop(
    on_event: &Channel<StreamEvent>,
    messages: &mut Vec<ChatMessage>,
    model: &str,
    tools: Vec<serde_json::Value>,
    think: bool,
    working_dir: PathBuf,
    cancel: CancellationToken,
) -> Result<u32, String> {
    let mut accumulated_tokens: u32 = 0;

    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }

        let request = build_request(model, messages, &tools, think);
        let result = ollama_stream::stream_chat(on_event, &request, cancel.clone()).await?;

        accumulated_tokens += result.eval_count + result.prompt_tokens;

        let assistant_msg = build_assistant_message(&result);
        messages.push(assistant_msg);

        if result.tool_calls.is_empty() {
            break;
        }

        if turn == MAX_TURNS - 1 {
            let _ = on_event.send(StreamEvent::Error {
                message: "Limite de tours atteinte".to_string(),
            });
            break;
        }

        let tool_results = tool_dispatcher::dispatch_multiple(
            &result.tool_calls,
            &working_dir,
        )
        .await;

        for (i, tr) in tool_results.iter().enumerate() {
            let (name, _): &(String, serde_json::Value) = &result.tool_calls[i];
            let _ = on_event.send(StreamEvent::ToolResult {
                name: name.clone(),
                content: tr.content.clone(),
                is_error: tr.is_error,
            });
            messages.push(ChatMessage {
                role: "tool".to_string(),
                content: tr.content.clone(),
                images: None,
                tool_calls: None,
                tool_name: Some(name.clone()),
            });
        }
    }

    decharge_gpu(model).await;
    Ok(accumulated_tokens)
}

fn build_request(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
) -> ChatRequest {
    let _ = think;
    ChatRequest {
        model: model.to_string(),
        messages: messages.to_vec(),
        stream: true,
        tools: if tools.is_empty() { None } else { Some(tools.to_vec()) },
        options: Some(ChatOptions { num_ctx: Some(32768) }),
        keep_alive: None,
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
                .map(|(name, args)| crate::services::agent_local::types_ollama::ToolCallOllama {
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
    }
}

async fn decharge_gpu(model: &str) {
    let client = reqwest::Client::new();
    let _ = client
        .post(format!("{BASE_URL}/api/chat"))
        .json(&serde_json::json!({
            "model": model,
            "messages": [],
            "keep_alive": "0"
        }))
        .send()
        .await;
}
