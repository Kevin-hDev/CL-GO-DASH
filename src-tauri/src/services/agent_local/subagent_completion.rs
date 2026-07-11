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
    persist_terminal_completion_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        status,
        summary,
        || async {},
        || async {},
    )
    .await
}

pub(super) async fn persist_instruction_delivery_failure(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
) -> Result<(), String> {
    let lock = super::session_store::lock_session(child_session_id).await;
    let _guard = lock.lock().await;
    let mut child = match super::session_store::get(child_session_id).await {
        Ok(child) => child,
        Err(_) => {
            super::subagent_completion_boundary::complete_missing(
                parent_session_id,
                child_session_id,
                subagent_type,
                || async {},
            )
            .await?;
            return Err(SUBAGENT_COMPLETION_ERROR.to_string());
        }
    };
    super::subagent_completion_boundary::complete_failure(
        parent_session_id,
        child_session_id,
        subagent_type,
        &mut child,
        true,
        true,
        true,
        || async {},
    )
    .await
}

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
        || async {},
        after_report,
    )
    .await
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
        after_child_loaded,
        after_report,
    )
    .await
}

async fn persist_terminal_completion_inner<FL, FLFut, FR, FRFut>(
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
    let lock = super::session_store::lock_session(child_session_id).await;
    let _guard = lock.lock().await;
    let mut child = match super::session_store::get(child_session_id).await {
        Ok(child) => child,
        Err(_) => {
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
        return Ok(finalized);
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
    Ok(finalized)
}
