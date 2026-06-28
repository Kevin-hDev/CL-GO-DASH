use crate::services::agent_local::permission_gate::{self, PermissionDecision};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_hooks::{run_post_hooks, run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use tokio_util::sync::CancellationToken;

use super::tool_executor_helpers::{
    check_write_guard, dispatch_or_interactive, post_record_read, push_tool_result,
};

pub async fn run_sequential(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
    plan_mode_active: bool,
) {
    for (idx, (name, args)) in tool_calls.iter().enumerate() {
        let arg_summary = super::tool_executor_diagnostics::started(
            session_id,
            request_id,
            name,
            args,
            working_dir,
        )
        .await;
        let plan_check = super::tool_plan_guard::ensure_allowed_for_session(
            name,
            args,
            session_id,
            plan_mode_active,
        )
        .await;
        if let Err(msg) = plan_check {
            let tr = super::tool_executor_plan::denied_with_summary(
                session_id,
                request_id,
                name,
                msg,
                arg_summary,
            )
            .await;
            push_tool_result(on_event, messages, name, tr, idx);
            continue;
        }
        match run_pre_hooks(name, args) {
            PreHookDecision::Deny(msg) => {
                let tr = tool_dispatcher::enrich_error(ToolResult::err(msg), name);
                super::tool_executor_diagnostics::completed(
                    session_id,
                    request_id,
                    name,
                    arg_summary,
                    true,
                )
                .await;
                push_tool_result(on_event, messages, name, tr, idx);
                continue;
            }
            PreHookDecision::Allow => {}
        }

        if let Err(msg) = check_write_guard(name, args, working_dir, write_guard) {
            let tr = tool_dispatcher::enrich_error(ToolResult::err(msg), name);
            super::tool_executor_diagnostics::completed(
                session_id,
                request_id,
                name,
                arg_summary,
                true,
            )
            .await;
            push_tool_result(on_event, messages, name, tr, idx);
            continue;
        }

        let allowed = check_allowed(on_event, name, args, session_id, cancel.clone()).await;
        let tr = if allowed {
            if let Err(msg) = super::tool_plan_guard::ensure_allowed_for_session(
                name,
                args,
                session_id,
                plan_mode_active,
            )
            .await
            {
                tool_dispatcher::enrich_error(ToolResult::err(msg), name)
            } else {
                dispatch_or_interactive(
                    on_event,
                    name,
                    args,
                    working_dir,
                    session_id,
                    cancel.clone(),
                )
                .await
            }
        } else {
            ToolResult::err("L'utilisateur a refusé cette action.")
        };

        let tr = run_post_hooks(name, args, tr);
        post_record_read(name, args, working_dir, &tr, write_guard);
        super::tool_executor_diagnostics::completed(
            session_id,
            request_id,
            name,
            arg_summary,
            tr.is_error,
        )
        .await;
        push_tool_result(on_event, messages, name, tr, idx);
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
