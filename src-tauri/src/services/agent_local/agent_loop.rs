use crate::services::agent_local::circuit_breaker;
use crate::services::agent_local::compress_hook;
use crate::services::agent_local::context_budget;
use crate::services::agent_local::eager_dispatch;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_executor;
use crate::services::agent_local::tool_result_budget;
use crate::services::agent_local::types_ollama::{ChatMessage, OllamaThink, StreamEvent};
use crate::services::agent_local::write_guard::WriteGuard;
use crate::services::agent_local::{
    agent_loop_errors, agent_loop_plan, agent_loop_support, ollama_stream,
};
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

    tool_result_budget::cleanup_old_results();
    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }

        tool_result_budget::apply_budget(messages);
        let _ = compress_hook::try_auto_compress(
            on_event,
            messages,
            model,
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
            agent_loop_plan::PlanLoopAction::Accept => {}
            agent_loop_plan::PlanLoopAction::Retry => {
                plan_repairs += 1;
                eager_handle.abort();
                continue;
            }
            agent_loop_plan::PlanLoopAction::Stop => {
                eager_handle.abort();
                break;
            }
        }
        messages.push(agent_loop_support::build_assistant_message(&result));

        if let Some(context_tokens) = compress_hook::try_auto_compress(
            on_event,
            messages,
            model,
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
            agent_loop_errors::max_turns(on_event, &session_id, &request_id).await;
            break;
        }

        if let Err(msg) = breaker.check(&result.tool_calls) {
            eager_handle.abort();
            agent_loop_errors::send(on_event, &session_id, &request_id, &msg).await;
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
            plan_active,
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
