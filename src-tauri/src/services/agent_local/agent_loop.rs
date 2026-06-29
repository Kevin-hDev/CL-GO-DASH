use super::agent_loop_compression::LoopCompression;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, OllamaThink, StreamEvent};
use super::write_guard_registry;
use super::{
    agent_loop_errors, agent_loop_limits::MAX_TURNS, agent_loop_plan, agent_loop_support,
    circuit_breaker, context_budget, eager_dispatch, ollama_stream, tool_executor,
    tool_result_budget,
};
use crate::services::token_counting;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
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
    plan_mode_active: bool,
) -> Result<u32, String> {
    let mut total_eval: Option<u32> = Some(0);
    let mut total_prompt: Option<u32> = Some(0);
    let mut last_prompt: Option<u32> = None;
    let mut last_eval: Option<u32> = None;
    let start = std::time::Instant::now();
    let mut breaker = circuit_breaker::CircuitBreaker::new();
    let write_guard_arc = write_guard_registry::lock(&session_id).await;
    let mut write_guard = write_guard_arc.lock().await;
    let mut plan_repairs = 0;
    let compression = LoopCompression {
        on_event,
        model,
        session_id: &session_id,
        request_id: &request_id,
        native_context,
        configured_context,
    };
    tool_result_budget::cleanup_old_results();
    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }

        tool_result_budget::apply_budget(messages);
        let _ = compression
            .try_run(messages, last_prompt, last_eval, cancel.clone())
            .await;
        context_budget::prepare_for_request(messages, configured_context);
        let plan_active = agent_loop_plan::active(&session_id, plan_mode_active).await;
        let request = agent_loop_support::build_request(model, messages, &tools, think.clone());
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
            plan_active,
        )
        .await?;
        token_counting::add_real_count(&mut total_eval, result.eval_count);
        token_counting::add_real_count(&mut total_prompt, result.prompt_tokens);
        last_prompt = result.prompt_tokens;
        last_eval = result.eval_count;
        match agent_loop_plan::check_result(
            on_event,
            messages,
            &session_id,
            &request_id,
            &result,
            plan_active,
            plan_repairs,
        )
        .await
        {
            agent_loop_plan::PlanLoopAction::Accept => plan_repairs = 0,
            agent_loop_plan::PlanLoopAction::Retry => {
                plan_repairs += 1;
                eager_handle.abort();
                continue;
            }
            agent_loop_plan::PlanLoopAction::Stop(message) => {
                eager_handle.abort();
                agent_loop_support::decharge_gpu(model).await;
                return Err(message.to_string());
            }
        }
        let mut assistant_message = agent_loop_support::build_assistant_message(&result);
        if plan_active && !result.tool_calls.is_empty() {
            assistant_message.content.clear();
        }
        messages.push(assistant_message);

        if compression
            .try_run(messages, last_prompt, last_eval, cancel.clone())
            .await
            .is_some()
        {
            last_prompt = None;
            last_eval = None;
        }

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
            agent_loop_support::decharge_gpu(model).await;
            return Err(agent_loop_errors::max_turns_message());
        }

        if let Err(msg) = breaker.check(&result.tool_calls) {
            eager_handle.abort();
            agent_loop_support::decharge_gpu(model).await;
            return Err(msg);
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
            plan_active,
            Some(eager_results),
        )
        .await;

        let compressed_after_tools = compression
            .try_run(messages, last_prompt, last_eval, cancel.clone())
            .await
            .is_some();
        if compressed_after_tools {
            last_prompt = None;
            last_eval = None;
        }

        if !compressed_after_tools {
            let _ = on_event.send(StreamEvent::TurnEnd {});
        }
    }

    let elapsed_ns = start.elapsed().as_nanos() as u64;
    let final_tps = if elapsed_ns > 0 {
        total_eval.unwrap_or(0) as f64 / (elapsed_ns as f64 / 1e9)
    } else {
        0.0
    };
    let _ = on_event.send(StreamEvent::Done {
        eval_count: total_eval,
        eval_duration_ns: elapsed_ns,
        final_tps,
        prompt_tokens: total_prompt,
        context_tokens: token_counting::sum_real_counts(last_prompt, last_eval),
    });

    super::stream_diagnostics::record_completed(&session_id, &request_id).await;
    agent_loop_support::decharge_gpu(model).await;
    Ok(token_counting::sum_real_counts(total_eval, total_prompt).unwrap_or(0))
}
