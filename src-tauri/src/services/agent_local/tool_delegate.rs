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
const MAX_NAME_SIZE: usize = 100;

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
    pub result_message: String,
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
            if trimmed.len() > MAX_PROMPT_SIZE {
                return Err(ToolResult::err(format!(
                    "Prompt trop long ({} chars, max {MAX_PROMPT_SIZE})",
                    trimmed.len()
                )));
            }
            trimmed.to_string()
        }
        _ => return Err(ToolResult::err("Paramètre 'prompt' manquant ou vide")),
    };
    let subagent_type = match args["subagent_type"].as_str() {
        Some("explorer") => "explorer",
        Some("coder") => "coder",
        Some(other) => {
            return Err(ToolResult::err(format!(
                "Type '{other}' invalide. Valeurs acceptées : 'explorer', 'coder'"
            )))
        }
        None => return Err(ToolResult::err("Paramètre 'subagent_type' manquant")),
    };
    let raw_name = args["name"].as_str().unwrap_or(subagent_type);
    let name: String = raw_name.chars().take(MAX_NAME_SIZE).collect();

    let parent = match session_store::get(&parent_session_id).await {
        Ok(s) => s,
        Err(_) => {
            return Err(ToolResult::err(
                "Erreur interne lors de la création du sous-agent",
            ))
        }
    };

    let run_id = subagent_registry::get_or_create_run_id(&parent_session_id).await;

    let child = match session_store::create_full(
        &name,
        &parent.model,
        &parent.provider,
        false,
        parent.project_id.clone(),
    )
    .await
    {
        Ok(mut s) => {
            s.parent_session_id = Some(parent_session_id.clone());
            s.subagent_type = Some(subagent_type.to_string());
            s.subagent_prompt = Some(prompt.clone());
            s.subagent_status = Some("running".to_string());
            s.subagent_run_id = Some(run_id.clone());
            if session_store::save(&s).await.is_err() {
                return Err(ToolResult::err(
                    "Erreur interne lors de la création du sous-agent",
                ));
            }
            s
        }
        Err(_) => {
            return Err(ToolResult::err(
                "Erreur interne lors de la création du sous-agent",
            ))
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
    let _ = session_store::add_messages(&child_id, vec![user_msg], 0).await;

    let cancel = CancellationToken::new();
    if let Err(e) = subagent_registry::register(
        &parent_session_id,
        &child_id,
        cancel.clone(),
        &name,
        subagent_type,
    )
    .await
    {
        return Err(ToolResult::err(e));
    }

    let prompt_preview: String = prompt.chars().take(MAX_PROMPT_PREVIEW).collect();
    let _ = parent_emitter.send(StreamEvent::SubagentSpawned {
        subagent_session_id: child_id.clone(),
        subagent_name: name.clone(),
        subagent_type: subagent_type.to_string(),
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
        result_message: format!(
            "Subagent '{name}' ({subagent_type}) spawned. Task: {prompt_preview}\n\
             The subagent is working autonomously. Do NOT perform this same work yourself. \
             Wait for the subagent report, then synthesize results for the user."
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::{MAX_NAME_SIZE, MAX_PROMPT_SIZE};

    // Vérification à la compilation que les bornes restent raisonnables.
    // (const assert évite le warning clippy::assertions_on_constants.)
    const _: () = {
        assert!(MAX_PROMPT_SIZE <= 100_000);
        assert!(MAX_NAME_SIZE <= 200);
    };
}
