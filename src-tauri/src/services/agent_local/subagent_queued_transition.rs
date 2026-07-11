use super::subagent_queued::PreparedFollowup;
use super::types_session::AgentMessage;
use std::future::Future;
use tokio_util::sync::CancellationToken;

pub(super) async fn prepare_next_followup(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
) -> Result<Option<PreparedFollowup>, String> {
    prepare_next_followup_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        || async {},
    )
    .await
}

#[cfg(test)]
pub(super) async fn prepare_next_followup_with_after_renew<F, Fut>(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    after_renew: F,
) -> Result<Option<PreparedFollowup>, String>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    prepare_next_followup_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        after_renew,
    )
    .await
}

async fn prepare_next_followup_inner<F, Fut>(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    after_renew: F,
) -> Result<Option<PreparedFollowup>, String>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let has_prompt = match has_next_prompt(child_session_id).await {
        Ok(value) => value,
        Err(error) => {
            return Err(terminate_with_error(
                parent_session_id,
                child_session_id,
                subagent_type,
                error,
            )
            .await)
        }
    };
    if !has_prompt {
        finalize_unstarted_followup(parent_session_id, child_session_id, subagent_type).await?;
        return Ok(None);
    }

    let cancel = CancellationToken::new();
    let run_id = match super::subagent_registry::renew_child(
        parent_session_id,
        child_session_id,
        cancel.clone(),
    )
    .await
    {
        Ok(run_id) => run_id,
        Err(error) => {
            return Err(terminate_with_error(
                parent_session_id,
                child_session_id,
                subagent_type,
                error,
            )
            .await)
        }
    };
    after_renew().await;
    let taken = match take_next_prompt(child_session_id, &run_id).await {
        Ok(value) => value,
        Err(error) => {
            return Err(terminate_with_error(
                parent_session_id,
                child_session_id,
                subagent_type,
                error,
            )
            .await)
        }
    };
    let Some((prompt, name, description, color_key)) = taken else {
        finalize_unstarted_followup(parent_session_id, child_session_id, subagent_type).await?;
        return Ok(None);
    };
    Ok(Some(PreparedFollowup {
        cancel,
        color_key,
        description,
        name,
        prompt,
        run_id,
    }))
}

pub(super) async fn finalize_unstarted_followup(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
) -> Result<(), String> {
    let result = super::subagent_completion::persist_unstarted_followup_failure(
        parent_session_id,
        child_session_id,
        subagent_type,
    )
    .await;
    super::subagent_working_dir::cleanup(child_session_id).await;
    super::session_store::remove_session_lock(child_session_id).await;
    result
}

async fn terminate_with_error(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    original_error: String,
) -> String {
    finalize_unstarted_followup(parent_session_id, child_session_id, subagent_type)
        .await
        .err()
        .unwrap_or(original_error)
}

async fn has_next_prompt(child_id: &str) -> Result<bool, String> {
    let lock = super::session_store::lock_session(child_id).await;
    let _guard = lock.lock().await;
    let child = super::session_store::get(child_id).await?;
    Ok(!child.subagent_queued_prompts.is_empty())
}

async fn take_next_prompt(
    child_id: &str,
    run_id: &str,
) -> Result<Option<(String, String, String, String)>, String> {
    let lock = super::session_store::lock_session(child_id).await;
    let _guard = lock.lock().await;
    let mut child = super::session_store::get(child_id).await?;
    if child.subagent_queued_prompts.is_empty() {
        return Ok(None);
    }
    let prompt = child.subagent_queued_prompts.remove(0);
    child.subagent_prompt = Some(prompt.clone());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.subagent_run_id = Some(run_id.to_string());
    child.subagent_summary = None;
    child.subagent_last_activity = Some(super::types_session::SubagentLastActivity {
        kind: "status".to_string(),
        label: "Relancé".to_string(),
        detail: Some(prompt.chars().take(220).collect()),
        updated_at: chrono::Utc::now(),
    });
    child.updated_at = Some(chrono::Utc::now());
    let name = child.name.clone();
    let description = child.subagent_description.clone().unwrap_or_default();
    let color_key = child.subagent_color_key.clone().unwrap_or_else(|| {
        super::subagent_profile::default_color_key(
            child.subagent_type.as_deref().unwrap_or("explorer"),
        )
        .to_string()
    });
    child.messages.push(AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: prompt.clone(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
    });
    super::session_store::save(&child).await?;
    Ok(Some((prompt, name, description, color_key)))
}
