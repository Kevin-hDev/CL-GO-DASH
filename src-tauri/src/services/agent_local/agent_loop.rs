use super::agent_loop_compression::{LastCounts, LoopCompression};
use super::agent_loop_ollama_request::OllamaRequestParams;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, OllamaThink, StreamEvent};
use super::write_guard_registry;
use super::{
    agent_loop_limits::MAX_TURNS, agent_loop_plan, agent_loop_support, circuit_breaker,
    subagent_orchestration, tool_executor, tool_result_budget,
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
    let (mut total_eval, mut total_prompt) = (Some(0), Some(0));
    let (mut last_prompt, mut last_eval) = (None, None);
    let start = std::time::Instant::now();
    let mut breaker = circuit_breaker::CircuitBreaker::new();
    let write_guard_arc = write_guard_registry::lock(&session_id).await;
    let mut write_guard = write_guard_arc.lock().await;
    let mut plan_repairs = 0;
    let mut subagents = subagent_orchestration::ParentSubagentOrchestrator::new(&session_id).await;
    let compression = LoopCompression {
        on_event,
        model,
        session_id: &session_id,
        request_id: &request_id,
        native_context,
        configured_context,
        working_dir: &working_dir,
    };
    tool_result_budget::cleanup_old_results();
    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }
        let request_output = super::agent_loop_ollama_request::run(OllamaRequestParams {
            on_event,
            messages,
            model,
            tools: &tools,
            think: &think,
            working_dir: &working_dir,
            session_id: &session_id,
            request_id: &request_id,
            cancel: cancel.clone(),
            configured_context,
            plan_mode_active,
            turn,
            subagents: &mut subagents,
        })
        .await?;
        let eager_handle = request_output.eager_handle;
        let interrupted = request_output.interrupted;
        let plan_active = request_output.plan_active;
        let result = request_output.result;
        if interrupted {
            super::stream_buffer::finalize_interrupted_content(on_event, &result, plan_active);
            eager_handle.abort();
            compression
                .handle_interrupted(
                    messages,
                    &result,
                    LastCounts::new(&mut last_prompt, &mut last_eval),
                    cancel.clone(),
                )
                .await?;
            continue;
        }
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
        subagents
            .finalize_content_phase(on_event, &result, plan_active)
            .await;
        let mut assistant_message = agent_loop_support::build_assistant_message(&result);
        if plan_active && !result.tool_calls.is_empty() {
            assistant_message.content.clear();
        }
        messages.push(assistant_message);
        compression
            .try_run_and_reset(messages, &mut last_prompt, &mut last_eval, cancel.clone())
            .await;
        if result.tool_calls.is_empty() {
            eager_handle.abort();
            if subagents
                .continue_after_no_tool_turn(on_event, messages, cancel.clone())
                .await?
            {
                continue;
            }
            break;
        }
        agent_loop_support::record_detected_tool_calls(
            &session_id,
            &request_id,
            &result.tool_calls,
            &working_dir,
        )
        .await;
        if turn == MAX_TURNS - 1 {
            eager_handle.abort();
            agent_loop_support::ensure_more_turns(turn, model).await?;
        }
        if let Err(msg) = breaker.check(&result.tool_calls) {
            eager_handle.abort();
            agent_loop_support::decharge_gpu(model).await;
            return Err(msg);
        }
        let eager_results = eager_handle.await.unwrap_or_default();
        let mode = permission_mode.to_string();
        let tool_compression = compression.tool_compression(
            token_counting::sum_real_counts(last_prompt, last_eval),
            cancel.clone(),
        );
        let compressed_during_tools = tool_executor::run_tools_with_eager(
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
            &[],
            Some(&tool_compression),
        )
        .await;
        let compressed_after_tools = compression
            .after_tools(
                messages,
                compressed_during_tools,
                &mut last_prompt,
                &mut last_eval,
                cancel.clone(),
            )
            .await;
        if !compressed_after_tools {
            let _ = on_event.send(StreamEvent::TurnEnd {});
        }
    }
    let token_total = super::agent_loop_completion::emit_done(
        on_event,
        total_eval,
        total_prompt,
        last_prompt,
        last_eval,
        start,
    );

    super::stream_diagnostics::record_completed(&session_id, &request_id).await;
    agent_loop_support::decharge_gpu(model).await;
    Ok(token_total)
}
