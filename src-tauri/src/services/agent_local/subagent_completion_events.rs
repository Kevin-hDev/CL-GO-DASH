use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;

pub(super) async fn persist_terminal(
    parent_id: &str,
    child_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
    run_id: &str,
    execution_id: &str,
    success: bool,
    emitter: Option<&AgentEventEmitter>,
) -> Result<Option<super::subagent_task::FinalizedSubagent>, String> {
    let after_report = terminal_callback(emitter, child_id, success, status, summary, run_id);
    super::subagent_completion::persist_terminal_completion_inner(
        parent_id,
        child_id,
        subagent_type,
        status,
        summary,
        Some((run_id, execution_id)),
        || async {},
        after_report,
    )
    .await
}

pub(super) async fn persist_instruction_failure(
    parent_id: &str,
    child_id: &str,
    subagent_type: &str,
    run_id: &str,
    execution_id: &str,
    emitter: Option<&AgentEventEmitter>,
) -> Result<bool, String> {
    let summary = super::subagent_completion::SUBAGENT_COMPLETION_ERROR;
    let after_report = terminal_callback(
        emitter,
        child_id,
        false,
        super::subagent_status::FAILED,
        summary,
        run_id,
    );
    super::subagent_completion::persist_instruction_delivery_failure_inner(
        parent_id,
        child_id,
        subagent_type,
        Some((run_id, execution_id)),
        after_report,
    )
    .await
}

fn terminal_callback(
    emitter: Option<&AgentEventEmitter>,
    child_id: &str,
    success: bool,
    status: &str,
    summary: &str,
    run_id: &str,
) -> impl FnOnce() -> std::future::Ready<()> {
    let emitter = emitter.cloned();
    let event = StreamEvent::SubagentCompleted {
        subagent_session_id: child_id.to_string(),
        success,
        status: status.to_string(),
        summary: summary.to_string(),
        run_id: Some(run_id.to_string()),
    };
    move || {
        if let Some(emitter) = emitter {
            let _ = emitter.send(event);
        }
        std::future::ready(())
    }
}
