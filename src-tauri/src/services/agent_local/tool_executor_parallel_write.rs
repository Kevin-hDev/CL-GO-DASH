use super::stream_events::AgentEventEmitter;
use super::tool_executor_write::execute_write;
use super::types_tools::ToolResult;
use super::write_guard::WriteGuard;
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn execute_tracked_write(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &Value,
    ctx: WriteExecContext<'_>,
) -> ToolResult {
    let summary =
        super::tool_executor_diagnostics::started(ctx.session_id, ctx.request_id, name, args, ctx.working_dir)
            .await;
    let result = execute_write(
        on_event,
        name,
        args,
        ctx.working_dir,
        ctx.mode,
        ctx.write_guard,
        ctx.session_id,
        ctx.cancel,
    )
    .await;
    super::tool_executor_diagnostics::completed(
        ctx.session_id,
        ctx.request_id,
        name,
        summary,
        result.is_error,
    )
    .await;
    result
}

pub struct WriteExecContext<'a> {
    pub working_dir: &'a Path,
    pub mode: &'a str,
    pub write_guard: &'a mut WriteGuard,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub cancel: CancellationToken,
}
