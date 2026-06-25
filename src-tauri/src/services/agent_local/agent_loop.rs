use crate::services::agent_local::circuit_breaker;
use crate::services::agent_local::compress_hook;
use crate::services::agent_local::context_budget;
use crate::services::agent_local::eager_dispatch;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_executor;
use crate::services::agent_local::tool_result_budget;
use crate::services::agent_local::types_ollama::{ChatMessage, OllamaThink, StreamEvent};
use crate::services::agent_local::write_guard::WriteGuard;
use crate::services::agent_local::{agent_loop_support, ollama_stream};
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

const MAX_TURNS: usize = 30;

pub async fn run_agent_loop(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    model: &str,
    tools: Vec<serde_json::Value>,
    think: OllamaThink,
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

    // Cleanup des résultats persistés > 24h (une fois par session)
    tool_result_budget::cleanup_old_results();

    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }

        tool_result_budget::apply_budget(messages);
        compress_hook::try_auto_compress(
            on_event,
            messages,
            model,
            &session_id,
            native_context,
            configured_context,
            last_prompt + last_eval,
            cancel.clone(),
        )
        .await;
        context_budget::prepare_for_request(messages, configured_context);
        let request = agent_loop_support::build_request(model, messages, &tools, think.clone());

        // Eager dispatch : lancer les read-only tools dès qu'ils arrivent dans le stream
        let (tool_tx, tool_rx) = tokio::sync::mpsc::unbounded_channel();
        let eager_working_dir = working_dir.clone();
        let eager_handle = tokio::spawn(eager_dispatch::collect_eager_results(
            tool_rx,
            eager_working_dir,
            session_id.clone(),
            request_id.clone(),
        ));

        super::stream_diagnostics::mark_phase(
            &session_id,
            &request_id,
            "model_stream",
            "Stream modèle démarré.",
        )
        .await;
        let result = ollama_stream::stream_chat_with_tool_notify(
            on_event,
            &request,
            cancel.clone(),
            tool_tx,
        )
        .await?;

        // Le sender est droppé quand le stream se termine → le receiver se ferme
        // et collect_eager_results peut retourner ses résultats.

        total_eval += result.eval_count;
        total_prompt += result.prompt_tokens;
        last_prompt = result.prompt_tokens;
        last_eval = result.eval_count;
        messages.push(agent_loop_support::build_assistant_message(&result));

        // Check post-réponse : compresser si le seuil a été dépassé pendant la génération
        compress_hook::try_auto_compress(
            on_event,
            messages,
            model,
            &session_id,
            native_context,
            configured_context,
            last_prompt + last_eval,
            cancel.clone(),
        )
        .await;

        if result.tool_calls.is_empty() {
            eager_handle.abort();
            break;
        }
        for (name, args) in &result.tool_calls {
            super::tool_executor_diagnostics::detected(
                &session_id,
                &request_id,
                name,
                args,
                &working_dir,
            )
            .await;
        }

        if turn == MAX_TURNS - 1 {
            eager_handle.abort();
            let diagnostic = super::stream_diagnostics::record_failure(
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
            eager_handle.abort();
            let diagnostic = super::stream_diagnostics::record_failure(
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

        let eager_results = eager_handle.await.unwrap_or_default();

        let mode = permission_mode.to_string();
        tool_executor::run_tools_with_eager(
            on_event,
            messages,
            &result.tool_calls,
            &working_dir,
            &mode,
            &session_id,
            &request_id,
            cancel.clone(),
            &mut write_guard,
            Some(eager_results),
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
        context_tokens: last_prompt + last_eval,
    });

    super::stream_diagnostics::record_completed(&session_id, &request_id).await;
    agent_loop_support::decharge_gpu(model).await;
    Ok(total_eval + total_prompt)
}
