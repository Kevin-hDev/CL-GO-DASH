use crate::services::agent_local::types_ollama::ChatMessage;
use std::path::Path;

pub(super) async fn record_detected_tool_calls(
    session_id: &str,
    request_id: &str,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &Path,
) {
    for (name, args) in tool_calls {
        crate::services::agent_local::tool_executor_diagnostics::detected(
            session_id,
            request_id,
            name,
            args,
            working_dir,
        )
        .await;
    }
}

pub(super) fn assign_tool_call_ids(
    messages: &mut [ChatMessage],
    before: usize,
    tool_call_ids: &[String],
) {
    let pushed = &mut messages[before..];
    for (i, msg) in pushed.iter_mut().enumerate() {
        if msg.role == "tool" {
            msg.tool_call_id = tool_call_ids.get(i).cloned();
        }
    }
}
