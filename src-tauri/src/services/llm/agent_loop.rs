//! Agent loop pour providers LLM API (OpenAI-compat).
//!
//! Miroir de `agent_local/agent_loop.rs` côté Ollama : boucle chat + tool_calls + exec
//! jusqu'à ce que le modèle n'appelle plus d'outil. Réutilise `tool_executor::run_tools`
//! pour dispatcher et gérer les permissions.

use super::stream;
use crate::services::agent_local::agent_settings;
use crate::services::agent_local::tool_executor;
use crate::services::agent_local::types_ollama::{
    ChatMessage, StreamEvent, StreamResult, ToolCallFunction, ToolCallOllama,
};
use std::path::PathBuf;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

const MAX_TURNS: usize = 50;

/// Les tool defs d'Ollama sont déjà au format OpenAI `{type: "function", function: {...}}`.
/// Cette fonction est l'identité — gardée pour lisibilité et future divergence.
pub fn convert_tools_to_openai(tools: &[serde_json::Value]) -> Vec<serde_json::Value> {
    tools.to_vec()
}

pub async fn run_agent_loop(
    on_event: &Channel<StreamEvent>,
    provider_id: &str,
    model: &str,
    messages: &mut Vec<ChatMessage>,
    tools: &[serde_json::Value],
    working_dir: PathBuf,
    session_id: String,
    cancel: CancellationToken,
) -> Result<u32, String> {
    let mut total_eval: u32 = 0;
    let mut total_prompt: u32 = 0;
    let start = std::time::Instant::now();

    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }

        let result =
            stream::stream_chat_no_done(on_event, provider_id, model, messages, tools, cancel.clone())
                .await?;

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

        // Snapshot longueur avant push des tool results → patch post-run.
        let before = messages.len();
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

        // Patch : assigne tool_call_id aux messages role:"tool" juste poussés,
        // dans l'ordre des tool_calls. Requis pour OpenAI-compat au tour suivant.
        let pushed = &mut messages[before..];
        for (i, msg) in pushed.iter_mut().enumerate() {
            if msg.role == "tool" {
                msg.tool_call_id = result.tool_call_ids.get(i).cloned();
            }
        }

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

    Ok(total_eval + total_prompt)
}

fn build_assistant_message(result: &StreamResult) -> ChatMessage {
    let tool_calls = if result.tool_calls.is_empty() {
        None
    } else {
        Some(
            result
                .tool_calls
                .iter()
                .enumerate()
                .map(|(i, (name, args))| ToolCallOllama {
                    id: result.tool_call_ids.get(i).cloned(),
                    function: ToolCallFunction {
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
