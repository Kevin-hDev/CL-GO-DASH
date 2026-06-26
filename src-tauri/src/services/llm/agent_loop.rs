use super::agent_loop_tools;
use super::compress_hook;
use super::retry;
use crate::services::agent_local::agent_loop_errors;
use crate::services::agent_local::agent_loop_limits::MAX_TURNS;
use crate::services::agent_local::agent_loop_plan;
use crate::services::agent_local::circuit_breaker;
use crate::services::agent_local::context_budget;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_executor;
use crate::services::agent_local::tool_result_budget;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::write_guard::WriteGuard;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

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
    plan_mode_active: bool,
) -> Result<u32, String> {
    let mut total_eval: u32 = 0;
    let mut total_prompt: u32 = 0;
    let mut last_prompt: u32 = 0;
    let mut last_eval: u32 = 0;
    let start = std::time::Instant::now();
    let mut breaker = circuit_breaker::CircuitBreaker::new();
    let mut write_guard = WriteGuard::new();
    let mut plan_repairs = 0;

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
        let plan_active = agent_loop_plan::active(&session_id, plan_mode_active).await;
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
            plan_active,
        )
        .await?;

        total_eval += result.eval_count;
        total_prompt += result.prompt_tokens;
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
        let mut assistant_message = super::agent_loop_message::build_assistant_message(&result);
        if plan_active && !result.tool_calls.is_empty() {
            assistant_message.content.clear();
        }
        messages.push(assistant_message);

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
            return Err(agent_loop_errors::max_turns_message());
        }

        breaker.check(&result.tool_calls)?;

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
            plan_active,
        )
        .await;

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
