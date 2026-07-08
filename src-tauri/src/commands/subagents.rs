use super::subagents_validation::validate_session_id;
use crate::services::agent_local::types_session::AgentSessionMeta;
use crate::services::agent_local::{
    session_store, session_subagents, subagent_live_state, subagent_registry,
};

#[cfg(test)]
pub use super::subagents_validation::validate_session_id_for_test;

#[tauri::command]
pub async fn list_subagents(
    parent_session_id: String,
    run_id: Option<String>,
) -> Result<Vec<AgentSessionMeta>, String> {
    validate_session_id(&parent_session_id)?;
    let all = session_store::list().await?;
    let mut visible = Vec::new();
    for item in all
        .into_iter()
        .filter(|s| s.parent_session_id.as_deref() == Some(&parent_session_id))
    {
        let normalized = subagent_live_state::normalize_meta(item).await;
        if run_id
            .as_ref()
            .is_none_or(|rid| normalized.subagent_run_id.as_deref() == Some(rid))
        {
            visible.push(normalized);
        }
    }
    Ok(visible)
}

#[tauri::command]
pub async fn cancel_subagent(subagent_session_id: String) -> Result<(), String> {
    validate_session_id(&subagent_session_id)?;
    if subagent_registry::cancel_one(&subagent_session_id).await {
        let _ = session_subagents::mark_status(&subagent_session_id, "cancelled").await;
        Ok(())
    } else {
        Err("Sous-agent introuvable ou déjà terminé".to_string())
    }
}
