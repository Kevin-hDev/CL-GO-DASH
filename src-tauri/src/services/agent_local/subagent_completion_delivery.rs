#[cfg(test)]
pub(super) async fn persist_instruction_delivery_failure(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
) -> Result<(), String> {
    persist_instruction_delivery_failure_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        None,
    )
    .await
    .map(|_| ())
}

pub(super) async fn persist_instruction_delivery_failure_for_execution(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    run_id: &str,
    execution_id: &str,
) -> Result<bool, String> {
    persist_instruction_delivery_failure_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        Some((run_id, execution_id)),
    )
    .await
}

async fn persist_instruction_delivery_failure_inner(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    expected_owner: Option<(&str, &str)>,
) -> Result<bool, String> {
    let lock = super::session_store::lock_session(child_session_id).await;
    let _guard = lock.lock().await;
    let mut child = match super::session_store::get(child_session_id).await {
        Ok(child) => child,
        Err(_) => {
            if !owns_missing_child(child_session_id, expected_owner).await {
                return Ok(false);
            }
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
    if !owns_loaded_child(&child, expected_owner).await {
        return Ok(false);
    }
    super::subagent_completion_boundary::complete_failure(
        parent_session_id,
        child_session_id,
        subagent_type,
        &mut child,
        true,
        true,
        || async {},
    )
    .await?;
    Ok(true)
}
