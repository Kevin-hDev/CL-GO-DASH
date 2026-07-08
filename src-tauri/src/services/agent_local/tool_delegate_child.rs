use crate::services::agent_local::session_store;
use crate::services::agent_local::types_session::AgentSession;
use crate::services::agent_local::types_tools::ToolResult;

pub(super) async fn prepare_existing_child(
    child_id: &str,
    parent_session_id: &str,
    subagent_type: &str,
    prompt: &str,
    name: &str,
    description: &str,
    color_key: &str,
    run_id: &str,
) -> Result<AgentSession, ToolResult> {
    if session_store::validate_session_id(child_id).is_err() {
        return Err(ToolResult::err("Sous-agent introuvable."));
    }
    let lock = session_store::lock_session(child_id).await;
    let _guard = lock.lock().await;
    let mut child = match session_store::get(child_id).await {
        Ok(session) => session,
        Err(_) => return Err(ToolResult::err("Sous-agent introuvable.")),
    };
    if child.parent_session_id.as_deref() != Some(parent_session_id) {
        return Err(ToolResult::err("Sous-agent introuvable."));
    }
    if super::subagent_live_state::has_pending_work(&child).await {
        return Err(ToolResult::err("Ce sous-agent est déjà en cours."));
    }
    child.name = name.to_string();
    child.subagent_type = Some(subagent_type.to_string());
    child.subagent_prompt = Some(prompt.to_string());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.subagent_run_id = Some(run_id.to_string());
    child.subagent_description = Some(description.to_string());
    child.subagent_color_key = Some(color_key.to_string());
    child.subagent_summary = None;
    session_store::save(&child)
        .await
        .map_err(|_| ToolResult::err("Erreur interne lors de la préparation du sous-agent"))?;
    Ok(child)
}

pub(super) async fn create_child(
    parent: &AgentSession,
    parent_session_id: &str,
    subagent_type: &str,
    prompt: &str,
    name: &str,
    description: &str,
    color_key: &str,
    run_id: &str,
) -> Result<AgentSession, ToolResult> {
    let mut child = session_store::create_full(
        name,
        &parent.model,
        &parent.provider,
        false,
        parent.project_id.clone(),
    )
    .await
    .map_err(|_| ToolResult::err("Erreur interne lors de la création du sous-agent"))?;
    child.parent_session_id = Some(parent_session_id.to_string());
    child.subagent_type = Some(subagent_type.to_string());
    child.subagent_prompt = Some(prompt.to_string());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.subagent_run_id = Some(run_id.to_string());
    child.subagent_description = Some(description.to_string());
    child.subagent_color_key = Some(color_key.to_string());
    session_store::save(&child)
        .await
        .map_err(|_| ToolResult::err("Erreur interne lors de la création du sous-agent"))?;
    Ok(child)
}
