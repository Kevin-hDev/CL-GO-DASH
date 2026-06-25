use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use tokio_util::sync::CancellationToken;

pub fn check_write_guard(
    name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    write_guard: &mut WriteGuard,
) -> Result<(), String> {
    if matches!(name, "write_file" | "edit_file") {
        let path_str = args["path"].as_str().unwrap_or("");
        let p = std::path::Path::new(path_str);
        let resolved = if p.is_absolute() {
            p.to_path_buf()
        } else {
            working_dir.join(p)
        };
        write_guard.check_write(&resolved)?;
    }
    Ok(())
}

pub fn post_record_read(
    name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    tr: &ToolResult,
    write_guard: &mut WriteGuard,
) {
    if name == "read_file" && !tr.is_error {
        if let Some(path_str) = args["path"].as_str() {
            let p = std::path::Path::new(path_str);
            let resolved = if p.is_absolute() {
                p.to_path_buf()
            } else {
                working_dir.join(p)
            };
            write_guard.record_read(&resolved);
        }
    }
}

pub fn push_tool_result(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    name: &str,
    tr: ToolResult,
    tool_call_index: usize,
) {
    let _ = on_event.send(StreamEvent::ToolResult {
        name: name.to_string(),
        content: tr.content.clone(),
        is_error: tr.is_error,
        truncated: tr.truncated,
        tool_call_index,
    });
    messages.push(ChatMessage {
        role: "tool".to_string(),
        content: tr.content,
        images: None,
        tool_calls: None,
        tool_name: Some(name.to_string()),
        tool_call_id: None,
        reasoning_content: None,
    });
}

pub async fn dispatch_or_interactive(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    if name == "ask_user_choice" {
        return super::tool_interactive::execute(args, on_event, cancel).await;
    }
    super::tool_dispatcher::dispatch(name, args, working_dir, session_id).await
}
