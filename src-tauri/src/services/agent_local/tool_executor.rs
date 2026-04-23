use crate::services::agent_local::permission_gate::{self, PermissionDecision};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_hooks::{run_post_hooks, run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use std::collections::HashMap;
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
    run_tools_with_eager(
        on_event, messages, tool_calls, working_dir, mode, session_id, cancel, write_guard, None,
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
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
    mut eager_results: Option<HashMap<usize, ToolResult>>,
) {
    if mode == "manual" {
        run_sequential(
            on_event, messages, tool_calls, working_dir, session_id, cancel, write_guard,
        )
        .await;
    } else {
        run_with_parallel_reads(
            on_event,
            messages,
            tool_calls,
            working_dir,
            cancel,
            write_guard,
            eager_results.as_mut(),
            session_id,
        )
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
        // Pre-hook : path traversal, fichiers sensibles, etc.
        let (blocked, effective_args_owned);
        match run_pre_hooks(name, args) {
            PreHookDecision::Deny(msg) => {
                let tr = tool_dispatcher::enrich_error(ToolResult::err(msg), name);
                push_tool_result(on_event, messages, name, tr);
                continue;
            }
            PreHookDecision::AllowModified(new_args) => {
                effective_args_owned = Some(new_args);
                blocked = false;
            }
            PreHookDecision::Allow => {
                effective_args_owned = None;
                blocked = false;
            }
        }
        let _ = blocked;
        let effective_args = effective_args_owned.as_ref().unwrap_or(args);

        if let Err(msg) = check_write_guard(name, effective_args, working_dir, write_guard) {
            let tr = tool_dispatcher::enrich_error(ToolResult::err(msg), name);
            push_tool_result(on_event, messages, name, tr);
            continue;
        }

        let allowed = check_allowed(on_event, name, effective_args, session_id, cancel.clone()).await;
        let tr = if allowed {
            tool_dispatcher::dispatch(name, effective_args, working_dir, session_id).await
        } else {
            ToolResult::err("L'utilisateur a refusé cette action.")
        };

        let tr = run_post_hooks(name, effective_args, tr);
        post_record_read(name, effective_args, working_dir, &tr, write_guard);
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
    if !permission_gate::requires_permission(name, args) {
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
