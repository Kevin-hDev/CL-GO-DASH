use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

use super::tool_executor_parallel::run_with_parallel_reads;
use super::tool_executor_sequential::run_sequential;

pub async fn run_tools(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    mode: &str,
    session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
    plan_mode_active: bool,
) {
    run_tools_with_eager(
        on_event,
        messages,
        tool_calls,
        working_dir,
        mode,
        session_id,
        request_id,
        cancel,
        write_guard,
        plan_mode_active,
        None,
    )
    .await;
}

pub async fn run_tools_with_eager(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    mode: &str,
    session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
    plan_mode_active: bool,
    mut eager_results: Option<HashMap<usize, ToolResult>>,
) {
    if mode == "manual" {
        run_sequential(
            on_event,
            messages,
            tool_calls,
            working_dir,
            session_id,
            request_id,
            cancel,
            write_guard,
            plan_mode_active,
        )
        .await;
    } else {
        run_with_parallel_reads(
            on_event,
            messages,
            tool_calls,
            working_dir,
            mode,
            cancel,
            write_guard,
            eager_results.as_mut(),
            session_id,
            request_id,
            plan_mode_active,
        )
        .await;
    }
}
