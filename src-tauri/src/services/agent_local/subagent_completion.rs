use super::subagent_registry::SubagentTerminalKind;
use super::subagent_task::FinalizedSubagent;

pub(super) const SUBAGENT_COMPLETION_ERROR: &str =
    "Le sous-agent n'a pas pu finaliser son rapport.";

pub(super) async fn persist_terminal_completion(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
) -> Result<FinalizedSubagent, String> {
    let finalized =
        match super::subagent_task::finalize_session_after_run(child_session_id, status, summary)
            .await
        {
            Ok(finalized) => finalized,
            Err(_) => {
                persist_failure_state(
                    parent_session_id,
                    child_session_id,
                    subagent_type,
                    true,
                )
                .await?;
                return Err(SUBAGENT_COMPLETION_ERROR.to_string());
            }
        };

    if finalized.queued_followup {
        return Ok(finalized);
    }

    if !append_report(
        parent_session_id,
        child_session_id,
        subagent_type,
        &finalized.session_status,
        summary,
    )
    .await
    {
        persist_failure_state(parent_session_id, child_session_id, subagent_type, false).await?;
        return Err(SUBAGENT_COMPLETION_ERROR.to_string());
    }

    finish_registry(child_session_id, true).await?;
    Ok(finalized)
}

async fn append_report(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
) -> bool {
    let name = super::session_store::get(child_session_id)
        .await
        .map(|session| session.name)
        .unwrap_or_else(|_| "agent".to_string());
    let report = super::subagent_hidden_reports::build_report(
        child_session_id.to_string(),
        name,
        subagent_type.to_string(),
        status.to_string(),
        summary.to_string(),
    );
    super::subagent_hidden_reports::append(parent_session_id, report)
        .await
        .is_ok()
}

async fn persist_failed_status(child_session_id: &str) -> Result<(), String> {
    super::session_subagents::mark_status(child_session_id, super::subagent_status::FAILED).await
}

async fn persist_failure_state(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    append_generic_report: bool,
) -> Result<(), String> {
    let status_result = persist_failed_status(child_session_id).await;
    let report_persisted = !append_generic_report
        || append_report(
            parent_session_id,
            child_session_id,
            subagent_type,
            super::subagent_status::FAILED,
            SUBAGENT_COMPLETION_ERROR,
        )
        .await;
    finish_registry(child_session_id, false).await?;
    status_result.map_err(|_| SUBAGENT_COMPLETION_ERROR.to_string())?;
    if !report_persisted {
        return Err(SUBAGENT_COMPLETION_ERROR.to_string());
    }
    Ok(())
}

async fn finish_registry(
    child_session_id: &str,
    report_persisted: bool,
) -> Result<(), String> {
    let kind = if report_persisted {
        SubagentTerminalKind::ReportPersisted
    } else {
        SubagentTerminalKind::ReportPersistenceFailed
    };
    super::subagent_registry::complete_child(child_session_id, kind).await
}
