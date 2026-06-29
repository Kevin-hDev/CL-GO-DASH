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
