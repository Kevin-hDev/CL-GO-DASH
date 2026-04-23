use crate::services::agent_local::permission_gate::{self, PermissionDecision};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use tokio_util::sync::CancellationToken;

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
    for (name, args) in tool_calls {
        // Pre-check : write guard avant tout dispatch
        if matches!(name.as_str(), "write_file" | "edit_file") {
            let path_str = args["path"].as_str().unwrap_or("");
            let p = std::path::Path::new(path_str);
            let resolved = if p.is_absolute() { p.to_path_buf() } else { working_dir.join(p) };
            if let Err(msg) = write_guard.check_write(&resolved) {
                push_tool_result(on_event, messages, name, ToolResult::err(msg));
                continue;
            }
        }

        let tr = if mode == "manual" {
            let allowed = check_allowed(on_event, name, args, session_id, cancel.clone()).await;
            if allowed {
                tool_dispatcher::dispatch(name, args, working_dir).await
            } else {
                ToolResult::err("L'utilisateur a refusé cette action.")
            }
        } else {
            tool_dispatcher::dispatch(name, args, working_dir).await
        };

        // Post-action : record read après exécution réussie
        if name == "read_file" && !tr.is_error {
            if let Some(path_str) = args["path"].as_str() {
                let p = std::path::Path::new(path_str);
                let resolved = if p.is_absolute() { p.to_path_buf() } else { working_dir.join(p) };
                write_guard.record_read(&resolved);
            }
        }

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

fn push_tool_result(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    name: &str,
    tr: ToolResult,
) {
    let _ = on_event.send(StreamEvent::ToolResult {
        name: name.to_string(),
        content: tr.content.clone(),
        is_error: tr.is_error,
        truncated: tr.truncated,
    });
    messages.push(ChatMessage {
        role: "tool".to_string(),
        content: tr.content,
        images: None,
        tool_calls: None,
        tool_name: Some(name.to_string()),
        tool_call_id: None,
    });
}
