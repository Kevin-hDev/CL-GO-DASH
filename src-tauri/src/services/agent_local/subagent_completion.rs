use super::subagent_task::FinalizedSubagent;

pub(super) const SUBAGENT_COMPLETION_ERROR: &str =
    "Le sous-agent n'a pas pu finaliser son rapport.";

#[cfg(test)]
pub(super) async fn persist_terminal_completion(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
) -> Result<FinalizedSubagent, String> {
    persist_terminal_completion_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        status,
        summary,
        None,
        || async {},
        || async {},
    )
    .await?
    .ok_or_else(|| SUBAGENT_COMPLETION_ERROR.to_string())
}

pub(super) async fn persist_terminal_completion_for_execution(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
    run_id: &str,
    execution_id: &str,
) -> Result<Option<FinalizedSubagent>, String> {
    persist_terminal_completion_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        status,
        summary,
        Some((run_id, execution_id)),
        || async {},
        || async {},
    )
    .await
}

include!("subagent_completion_delivery.rs");

#[cfg(test)]
pub(super) async fn persist_terminal_completion_with_after_report<F, Fut>(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
    after_report: F,
) -> Result<FinalizedSubagent, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    persist_terminal_completion_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        status,
        summary,
        None,
        || async {},
        after_report,
    )
    .await?
    .ok_or_else(|| SUBAGENT_COMPLETION_ERROR.to_string())
}

#[cfg(test)]
pub(super) async fn persist_terminal_completion_with_hooks<FL, FLFut, FR, FRFut>(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
    after_child_loaded: FL,
    after_report: FR,
) -> Result<FinalizedSubagent, String>
where
    FL: FnOnce() -> FLFut,
    FLFut: std::future::Future<Output = ()>,
    FR: FnOnce() -> FRFut,
    FRFut: std::future::Future<Output = ()>,
{
    persist_terminal_completion_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        status,
        summary,
        None,
        after_child_loaded,
        after_report,
    )
    .await?
    .ok_or_else(|| SUBAGENT_COMPLETION_ERROR.to_string())
}

async fn persist_terminal_completion_inner<FL, FLFut, FR, FRFut>(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
    expected_owner: Option<(&str, &str)>,
    after_child_loaded: FL,
    after_report: FR,
) -> Result<Option<FinalizedSubagent>, String>
where
    FL: FnOnce() -> FLFut,
    FLFut: std::future::Future<Output = ()>,
    FR: FnOnce() -> FRFut,
    FRFut: std::future::Future<Output = ()>,
{
    let lock = super::session_store::lock_session(child_session_id).await;
    let _guard = lock.lock().await;
    let mut child = match super::session_store::get(child_session_id).await {
        Ok(child) => child,
        Err(_) => {
            if !owns_missing_child(child_session_id, expected_owner).await {
                return Ok(None);
            }
            super::subagent_completion_boundary::complete_missing(
                parent_session_id,
                child_session_id,
                subagent_type,
                after_report,
            )
            .await?;
            return Err(SUBAGENT_COMPLETION_ERROR.to_string());
        }
    };
    if !owns_loaded_child(&child, expected_owner).await {
        return Ok(None);
    }
    after_child_loaded().await;
    let finalized =
        super::subagent_task::finalize_loaded_session_after_run(&mut child, status, summary);
    if super::session_store::save(&child).await.is_err() {
        super::subagent_completion_boundary::complete_failure(
            parent_session_id,
            child_session_id,
            subagent_type,
            &mut child,
            true,
            false,
            false,
            after_report,
        )
        .await?;
        return Err(SUBAGENT_COMPLETION_ERROR.to_string());
    }

    if finalized.queued_followup {
        return Ok(Some(finalized));
    }

    super::subagent_completion_boundary::complete_success(
        parent_session_id,
        child_session_id,
        subagent_type,
        &mut child,
        &finalized.session_status,
        summary,
        after_report,
    )
    .await?;
    Ok(Some(finalized))
}

async fn owns_loaded_child(
    child: &super::types_session::AgentSession,
    expected_owner: Option<(&str, &str)>,
) -> bool {
    let Some((run_id, execution_id)) = expected_owner else {
        return true;
    };
    child.subagent_run_id.as_deref() == Some(run_id)
        && super::subagent_registry::owns_execution(&child.id, run_id, execution_id).await
}

async fn owns_missing_child(child_id: &str, expected_owner: Option<(&str, &str)>) -> bool {
    let Some((run_id, execution_id)) = expected_owner else {
        return true;
    };
    super::subagent_registry::owns_execution(child_id, run_id, execution_id).await
}
