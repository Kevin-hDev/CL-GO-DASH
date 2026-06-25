//! Miroir de `agent_local/agent_loop.rs` côté Ollama : boucle chat + tool_calls + exec
//! jusqu'à ce que le modèle n'appelle plus d'outil. Réutilise `tool_executor::run_tools`
//! pour dispatcher et gérer les permissions.

use super::agent_loop_tools;
use super::compress_hook;
use super::retry;
use crate::services::agent_local::circuit_breaker;
use crate::services::agent_local::context_budget;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_executor;
use crate::services::agent_local::tool_result_budget;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::write_guard::WriteGuard;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

const MAX_TURNS: usize = 30;

/// Les tool defs d'Ollama sont déjà au format OpenAI `{type: "function", function: {...}}`.
/// Cette fonction est l'identité — gardée pour lisibilité et future divergence.
pub fn convert_tools_to_openai(tools: &[serde_json::Value]) -> Vec<serde_json::Value> {
    tools.to_vec()
}

pub async fn run_agent_loop(
    on_event: &AgentEventEmitter,
    provider_id: &str,
    model: &str,
    messages: &mut Vec<ChatMessage>,
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    working_dir: PathBuf,
    session_id: String,
    request_id: String,
    cancel: CancellationToken,
    native_context: u64,
    configured_context: u64,
    permission_mode: &str,
) -> Result<u32, String> {
    let mut total_eval: u32 = 0;
    let mut total_prompt: u32 = 0;
    let mut last_prompt: u32 = 0;
    let mut last_eval: u32 = 0;
    let start = std::time::Instant::now();
    let mut breaker = circuit_breaker::CircuitBreaker::new();
    let mut write_guard = WriteGuard::new();

    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }

        tool_result_budget::apply_budget(messages);
        let _ = compress_hook::try_auto_compress(
            on_event,
            provider_id,
            model,
            messages,
            &session_id,
            &request_id,
            native_context,
            configured_context,
            last_prompt + last_eval,
            cancel.clone(),
        )
        .await;
        context_budget::prepare_for_request(messages, configured_context);
        crate::services::agent_local::stream_diagnostics::mark_phase(
            &session_id,
            &request_id,
            "model_stream",
            "Stream modèle démarré.",
        )
        .await;
        let result = retry::retry_stream(
            on_event,
            &session_id,
            &request_id,
            provider_id,
            model,
            messages,
            tools,
            think,
            reasoning_mode,
            cancel.clone(),
        )
        .await?;

        total_eval += result.eval_count;
        total_prompt += result.prompt_tokens;
        last_prompt = result.prompt_tokens;
        last_eval = result.eval_count;
        messages.push(super::agent_loop_message::build_assistant_message(&result));

        // Check post-réponse : compresser si le seuil a été dépassé pendant la génération
        if let Some(context_tokens) = compress_hook::try_auto_compress(
            on_event,
            provider_id,
            model,
            messages,
            &session_id,
            &request_id,
            native_context,
            configured_context,
            last_prompt + last_eval,
            cancel.clone(),
        )
        .await
        {
            last_prompt = 0;
            last_eval = context_tokens;
        }

        if result.tool_calls.is_empty() {
            break;
        }
        agent_loop_tools::record_detected_tool_calls(
            &session_id,
            &request_id,
            &result.tool_calls,
            &working_dir,
        )
        .await;

        if turn == MAX_TURNS - 1 {
            let diagnostic = crate::services::agent_local::stream_diagnostics::record_failure(
                &session_id,
                Some(&request_id),
                "Limite de tours atteinte",
                false,
            )
            .await;
            let _ = on_event.send(StreamEvent::Error {
                message: "Limite de tours atteinte".to_string(),
                is_connection: false,
                diagnostic,
            });
            break;
        }

        if let Err(msg) = breaker.check(&result.tool_calls) {
            let diagnostic = crate::services::agent_local::stream_diagnostics::record_failure(
                &session_id,
                Some(&request_id),
                &msg,
                false,
            )
            .await;
            let _ = on_event.send(StreamEvent::Error {
                message: msg,
                is_connection: false,
                diagnostic,
            });
            break;
        }

        // Snapshot longueur avant push des tool results → patch post-run.
        let before = messages.len();
        let mode = permission_mode.to_string();
        tool_executor::run_tools(
            on_event,
            messages,
            &result.tool_calls,
            &working_dir,
            &mode,
            &session_id,
            &request_id,
            cancel.clone(),
            &mut write_guard,
        )
        .await;

        // Patch : assigne tool_call_id aux messages role:"tool" juste poussés,
        // dans l'ordre des tool_calls. Requis pour OpenAI-compat au tour suivant.
        agent_loop_tools::assign_tool_call_ids(messages, before, &result.tool_call_ids);

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
        context_tokens: last_prompt + last_eval,
    });

    crate::services::agent_local::stream_diagnostics::record_completed(&session_id, &request_id)
        .await;
    Ok(total_eval + total_prompt)
}
