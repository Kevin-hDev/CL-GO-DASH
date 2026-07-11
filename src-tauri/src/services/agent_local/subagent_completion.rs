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
    let lock = super::session_store::lock_session(child_session_id).await;
    let _guard = lock.lock().await;
    let mut child = match super::session_store::get(child_session_id).await {
        Ok(child) => child,
        Err(_) => {
            let _report_persisted = append_report(
                parent_session_id,
                child_session_id,
                subagent_type,
                super::subagent_status::FAILED,
                SUBAGENT_COMPLETION_ERROR,
            )
            .await;
            finish_registry(parent_session_id, child_session_id, false).await?;
            return Err(SUBAGENT_COMPLETION_ERROR.to_string());
        }
    };
    let finalized =
        super::subagent_task::finalize_loaded_session_after_run(&mut child, status, summary);
    if super::session_store::save(&child).await.is_err() {
        persist_locked_failure(
            parent_session_id,
            child_session_id,
            subagent_type,
            &mut child,
            true,
        )
        .await?;
        return Err(SUBAGENT_COMPLETION_ERROR.to_string());
    }

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
        persist_locked_failure(
            parent_session_id,
            child_session_id,
            subagent_type,
            &mut child,
            false,
        )
        .await?;
        return Err(SUBAGENT_COMPLETION_ERROR.to_string());
    }

    finish_registry(parent_session_id, child_session_id, true).await?;
    Ok(finalized)
}

pub(super) async fn persist_unstarted_followup_failure(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
) -> Result<(), String> {
    persist_terminal_completion(
        parent_session_id,
        child_session_id,
        subagent_type,
        super::subagent_status::FAILED,
        SUBAGENT_COMPLETION_ERROR,
    )
    .await
    .map(|_| ())
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

async fn persist_locked_failure(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    child: &mut super::types_session::AgentSession,
    append_generic_report: bool,
) -> Result<(), String> {
    super::subagent_task::finalize_loaded_session_after_run(
        child,
        super::subagent_status::FAILED,
        SUBAGENT_COMPLETION_ERROR,
    );
    let status_result = super::session_store::save(child).await;
    let report_persisted = !append_generic_report
        || append_report(
            parent_session_id,
            child_session_id,
            subagent_type,
            super::subagent_status::FAILED,
            SUBAGENT_COMPLETION_ERROR,
        )
        .await;
    finish_registry(parent_session_id, child_session_id, false).await?;
    status_result.map_err(|_| SUBAGENT_COMPLETION_ERROR.to_string())?;
    if !report_persisted {
        return Err(SUBAGENT_COMPLETION_ERROR.to_string());
    }
    Ok(())
}

async fn finish_registry(
    parent_session_id: &str,
    child_session_id: &str,
    report_persisted: bool,
) -> Result<(), String> {
    let kind = if report_persisted {
        SubagentTerminalKind::ReportPersisted
    } else {
        SubagentTerminalKind::ReportPersistenceFailed
    };
    super::subagent_registry::complete_child_or_parent(
        parent_session_id,
        child_session_id,
        kind,
    )
    .await
}
