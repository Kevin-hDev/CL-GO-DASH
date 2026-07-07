use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_registry;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

const MAX_PROMPT_PREVIEW: usize = 120;
const MAX_PROMPT_SIZE: usize = 50_000;

pub struct SpawnedSubagent {
    pub app: AppHandle,
    pub child_id: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub subagent_type: String,
    pub parent_emitter: AgentEventEmitter,
    pub cancel: CancellationToken,
    pub project_id: Option<String>,
}

pub async fn prepare_delegate(
    args: Value,
    app: AppHandle,
    parent_session_id: String,
    parent_emitter: AgentEventEmitter,
) -> Result<SpawnedSubagent, ToolResult> {
    let prompt = match args["prompt"].as_str() {
        Some(p) if !p.trim().is_empty() => {
            let trimmed = p.trim();
            if trimmed.chars().count() > MAX_PROMPT_SIZE {
                return Err(ToolResult::err("Prompt sous-agent trop long."));
            }
            trimmed.to_string()
        }
        _ => return Err(ToolResult::err("Paramètre 'prompt' manquant ou vide")),
    };
    let subagent_type = match args["subagent_type"].as_str() {
        Some("explorer") => "explorer",
        Some("coder") => "coder",
        Some(_) => return Err(ToolResult::err("Type de sous-agent invalide.")),
        None => return Err(ToolResult::err("Paramètre 'subagent_type' manquant")),
    };
    let name = super::subagent_profile::clean_name(
        args["display_name"]
            .as_str()
            .or_else(|| args["name"].as_str()),
        subagent_type,
    );
    let description =
        super::subagent_profile::clean_description(args["description"].as_str(), &prompt);
    let color_key = super::subagent_profile::default_color_key(subagent_type).to_string();

    let parent = match session_store::get(&parent_session_id).await {
        Ok(s) => s,
        Err(_) => {
            return Err(ToolResult::err(
                "Erreur interne lors de la création du sous-agent",
            ))
        }
    };

    let run_id = subagent_registry::get_or_create_run_id(&parent_session_id).await;

    let child = match args["subagent_id"].as_str() {
        Some(id) if !id.trim().is_empty() => {
            match prepare_existing_child(
                id.trim(),
                &parent_session_id,
                subagent_type,
                &prompt,
                &name,
                &description,
                &color_key,
                &run_id,
            )
            .await
            {
                Ok(session) => session,
                Err(result) => return Err(result),
            }
        }
        _ => {
            match create_child(
                &parent,
                &parent_session_id,
                subagent_type,
                &prompt,
                &name,
                &description,
                &color_key,
                &run_id,
            )
            .await
            {
                Ok(session) => session,
                Err(result) => return Err(result),
            }
        }
    };

    let child_id = child.id.clone();

    let user_msg = crate::services::agent_local::types_session::AgentMessage {
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
    };
    if let Err(e) = session_store::add_messages(&child_id, vec![user_msg], 0).await {
        // Fail closed : ne pas démarrer un sous-agent dont le prompt n'est
        // pas persisté. On nettoie la session enfant créée plus haut.
        eprintln!("[subagent] persistance prompt enfant {}: {e}", child_id);
        let _ =
            super::session_subagents::mark_status(&child_id, super::subagent_status::FAILED).await;
        return Err(ToolResult::err(
            "Erreur interne lors de la création du sous-agent",
        ));
    }

    let cancel = CancellationToken::new();
    if let Err(e) = subagent_registry::register(&parent_session_id, &child_id, cancel.clone()).await
    {
        let _ =
            super::session_subagents::mark_status(&child_id, super::subagent_status::FAILED).await;
        return Err(ToolResult::err(e));
    }

    let prompt_preview: String = prompt.chars().take(MAX_PROMPT_PREVIEW).collect();
    let _ = parent_emitter.send(StreamEvent::SubagentSpawned {
        subagent_session_id: child_id.clone(),
        subagent_name: name.clone(),
        subagent_type: subagent_type.to_string(),
        subagent_description: description,
        subagent_color_key: color_key,
        prompt_preview: prompt_preview.clone(),
        run_id: Some(run_id),
    });

    Ok(SpawnedSubagent {
        app,
        child_id,
        model: parent.model.clone(),
        provider: parent.provider.clone(),
        prompt,
        subagent_type: subagent_type.to_string(),
        parent_emitter,
        cancel,
        project_id: parent.project_id.clone(),
    })
}

async fn prepare_existing_child(
    child_id: &str,
    parent_session_id: &str,
    subagent_type: &str,
    prompt: &str,
    name: &str,
    description: &str,
    color_key: &str,
    run_id: &str,
) -> Result<crate::services::agent_local::types_session::AgentSession, ToolResult> {
    let mut child = match session_store::get(child_id).await {
        Ok(session) => session,
        Err(_) => return Err(ToolResult::err("Sous-agent introuvable.")),
    };
    if child.parent_session_id.as_deref() != Some(parent_session_id) {
        return Err(ToolResult::err("Sous-agent introuvable."));
    }
    if child.subagent_status.as_deref() == Some(super::subagent_status::RUNNING) {
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

async fn create_child(
    parent: &crate::services::agent_local::types_session::AgentSession,
    parent_session_id: &str,
    subagent_type: &str,
    prompt: &str,
    name: &str,
    description: &str,
    color_key: &str,
    run_id: &str,
) -> Result<crate::services::agent_local::types_session::AgentSession, ToolResult> {
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

#[cfg(test)]
mod tests {
    use super::MAX_PROMPT_SIZE;

    // Vérification à la compilation que les bornes restent raisonnables.
    // (const assert évite le warning clippy::assertions_on_constants.)
    const _: () = {
        assert!(MAX_PROMPT_SIZE <= 100_000);
    };
}
