use crate::services::agent_local::permission_gate::{self, PermissionDecision};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use tokio_util::sync::CancellationToken;

use super::tool_executor_helpers::{check_write_guard, post_record_read, push_tool_result};
use super::tool_executor_parallel::run_with_parallel_reads;

pub async fn run_tools(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    mode: &str,
    session_id: &str,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
) {
    if mode == "manual" {
        run_sequential(
            on_event, messages, tool_calls, working_dir, session_id, cancel, write_guard,
        )
        .await;
    } else {
        run_with_parallel_reads(on_event, messages, tool_calls, working_dir, cancel, write_guard)
            .await;
    }
}

/// Mode manuel : permission demandée un par un, exécution séquentielle.
async fn run_sequential(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    session_id: &str,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
) {
    for (name, args) in tool_calls {
        if let Err(msg) = check_write_guard(name, args, working_dir, write_guard) {
            let tr = tool_dispatcher::enrich_error(ToolResult::err(msg), name);
            push_tool_result(on_event, messages, name, tr);
            continue;
        }

        let allowed = check_allowed(on_event, name, args, session_id, cancel.clone()).await;
        let tr = if allowed {
            tool_dispatcher::dispatch(name, args, working_dir).await
        } else {
            ToolResult::err("L'utilisateur a refusé cette action.")
        };

        post_record_read(name, args, working_dir, &tr, write_guard);
        push_tool_result(on_event, messages, name, tr);
    }
}

async fn check_allowed(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &serde_json::Value,
    session_id: &str,
    cancel: CancellationToken,
) -> bool {
    if !permission_gate::requires_permission(name) {
        return true;
    }
    if permission_gate::is_allowed(session_id, name).await {
        return true;
    }
    match permission_gate::request(on_event, name, args, cancel).await {
        PermissionDecision::Allow => true,
        PermissionDecision::AllowSession => {
            permission_gate::mark_allowed(session_id, name).await;
            true
        }
        PermissionDecision::Deny => false,
    }
}
