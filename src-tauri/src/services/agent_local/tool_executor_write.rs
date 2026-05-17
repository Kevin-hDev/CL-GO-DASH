use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_hooks::{run_post_hooks, run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use crate::services::agent_local::{permission_gate, permission_policy, sensitive_data};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::tool_executor_helpers::check_write_guard;

pub(super) async fn execute_write(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &Value,
    working_dir: &std::path::Path,
    mode: &str,
    write_guard: &mut WriteGuard,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    match run_pre_hooks(name, args) {
        PreHookDecision::Deny(msg) => {
            return tool_dispatcher::enrich_error(ToolResult::err(msg), name);
        }
        PreHookDecision::Allow => {}
    }

    if must_prompt_for_sensitive_bash(name, args) {
        let safe_args = sensitive_data::redact_json(args);
        if !request_once(on_event, name, &safe_args, cancel.clone()).await {
            return ToolResult::err("L'utilisateur a refusé cette action.");
        }
    } else if permission_policy::uses_auto_bypass(mode) {
        if !permission_policy::check_data_dir_write(
            on_event,
            name,
            args,
            working_dir,
            cancel.clone(),
        )
        .await
        {
            return ToolResult::err("L'utilisateur a refusé cette action.");
        }
    } else if permission_gate::requires_permission(name, args)
        && !permission_gate::is_allowed(session_id, name).await
    {
        match permission_gate::request(on_event, name, args, cancel.clone()).await {
            permission_gate::PermissionDecision::Allow => {}
            permission_gate::PermissionDecision::AllowSession => {
                permission_gate::mark_allowed(session_id, name).await;
            }
            permission_gate::PermissionDecision::Deny => {
                return ToolResult::err("L'utilisateur a refusé cette action.");
            }
        }
    }

    let tr = match check_write_guard(name, args, working_dir, write_guard) {
        Err(msg) => tool_dispatcher::enrich_error(ToolResult::err(msg), name),
        Ok(()) => {
            if cancel.is_cancelled() {
                ToolResult::err("Annulé.")
            } else {
                tool_dispatcher::dispatch(name, args, working_dir, session_id).await
            }
        }
    };
    run_post_hooks(name, args, tr)
}

fn must_prompt_for_sensitive_bash(name: &str, args: &Value) -> bool {
    name == "bash"
        && args["command"]
            .as_str()
            .map(sensitive_data::bash_touches_sensitive_data)
            .unwrap_or(false)
}

async fn request_once(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &Value,
    cancel: CancellationToken,
) -> bool {
    match permission_gate::request(on_event, name, args, cancel).await {
        permission_gate::PermissionDecision::Allow
        | permission_gate::PermissionDecision::AllowSession => true,
        permission_gate::PermissionDecision::Deny => false,
    }
}
