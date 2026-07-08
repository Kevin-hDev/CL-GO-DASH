use super::agent_loop_compression::{LastCounts, LoopCompression};
use super::agent_loop_tools;
use super::retry;
use crate::services::agent_local::agent_loop_errors;
use crate::services::agent_local::agent_loop_limits::MAX_TURNS;
use crate::services::agent_local::agent_loop_plan;
use crate::services::agent_local::circuit_breaker;
use crate::services::agent_local::context_budget;
use crate::services::agent_local::stream_diagnostics_model as model_diag;
use crate::services::agent_local::stream_diagnostics_payload as payload_diag;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_orchestration;
use crate::services::agent_local::tool_executor;
use crate::services::agent_local::tool_result_budget;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::write_guard_registry;
use crate::services::token_counting;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

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
        provider_id,
        model,
        session_id: &session_id,
        request_id: &request_id,
        native_context,
        configured_context,
        working_dir: &working_dir,
    };
    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }
        subagents.inject_pending_reports(messages).await;
        tool_result_budget::apply_budget(messages);
        context_budget::prepare_for_request(messages, configured_context);
        let realtime_budget = compression.realtime_budget(messages);
        let plan_active = agent_loop_plan::active(&session_id, plan_mode_active).await;
        model_diag::record_model_request(&session_id, &request_id, turn, messages).await;
        payload_diag::record_api_payload(&session_id, &request_id, turn, provider_id, messages)
            .await;
        crate::services::agent_local::stream_diagnostics::mark_phase(
            &session_id,
            &request_id,
            "model_stream",
            "Stream modèle démarré.",
        )
        .await;
        let outcome = retry::retry_stream(
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
            plan_active,
            realtime_budget,
        )
        .await?;
        let interrupted = outcome.is_interrupted();
        let result = outcome.into_result();
        model_diag::record_model_result(&session_id, &request_id, turn, &result).await;
        if interrupted {
            crate::services::agent_local::stream_buffer::finalize_interrupted_content(
                on_event,
                &result,
                plan_active,
            );
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
                continue;
            }
            agent_loop_plan::PlanLoopAction::Stop(message) => return Err(message.to_string()),
        }
        subagents
            .finalize_content_phase(on_event, &result, plan_active)
            .await;
        let mut assistant_message = super::agent_loop_message::build_assistant_message(&result);
        if plan_active && !result.tool_calls.is_empty() {
            assistant_message.content.clear();
        }
        messages.push(assistant_message);
        compression
            .try_run_and_reset(messages, &mut last_prompt, &mut last_eval, cancel.clone())
            .await;
        if result.tool_calls.is_empty() {
            if subagents
                .continue_after_no_tool_turn(on_event, messages, cancel.clone())
                .await?
            {
                continue;
            }
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
            return Err(agent_loop_errors::max_turns_message());
        }
        breaker.check(&result.tool_calls)?;
        let mode = permission_mode.to_string();
        let tool_compression = compression.tool_compression(
            token_counting::sum_real_counts(last_prompt, last_eval),
            cancel.clone(),
        );
        let compressed_during_tools = tool_executor::run_tools(
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
            &result.tool_call_ids,
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
    let token_total = crate::services::agent_local::agent_loop_completion::emit_done(
        on_event,
        total_eval,
        total_prompt,
        last_prompt,
        last_eval,
        start,
    );
    crate::services::agent_local::stream_diagnostics::record_completed(&session_id, &request_id)
        .await;
    Ok(token_total)
}
