use crate::services::agent_local::permission_gate::{self, PermissionDecision};
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::types_tools::ToolResult;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

pub async fn run_tools(
    on_event: &Channel<StreamEvent>,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    mode: &str,
    session_id: &str,
    cancel: CancellationToken,
) {
    if mode != "manual" {
        let results = tool_dispatcher::dispatch_multiple(tool_calls, working_dir).await;
        for (i, tr) in results.into_iter().enumerate() {
            let name = &tool_calls[i].0;
            push_tool_result(on_event, messages, name, tr);
        }
        return;
    }

    for (name, args) in tool_calls {
        let allowed = check_allowed(on_event, name, args, session_id, cancel.clone()).await;
        let tr = if allowed {
            tool_dispatcher::dispatch(name, args, working_dir).await
        } else {
            ToolResult {
                content: "L'utilisateur a refusé cette action.".to_string(),
                is_error: true,
            }
        };
        push_tool_result(on_event, messages, name, tr);
    }
}

async fn check_allowed(
    on_event: &Channel<StreamEvent>,
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
    on_event: &Channel<StreamEvent>,
    messages: &mut Vec<ChatMessage>,
    name: &str,
    tr: ToolResult,
) {
    let _ = on_event.send(StreamEvent::ToolResult {
        name: name.to_string(),
        content: tr.content.clone(),
        is_error: tr.is_error,
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
