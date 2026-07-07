use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::agent_local::types_session::AgentMessage;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

const MAX_PROMPT_PREVIEW: usize = 120;

pub struct QueuedSubagentRun {
    pub app: AppHandle,
    pub parent_session_id: String,
    pub child_session_id: String,
    pub model: String,
    pub provider: String,
    pub subagent_type: String,
    pub parent_emitter: AgentEventEmitter,
    pub project_id: Option<String>,
}

pub async fn spawn_next_if_present(params: QueuedSubagentRun) -> Result<bool, String> {
    let Some((prompt, name, description, color_key)) =
        take_next_prompt(&params.child_session_id).await?
    else {
        return Ok(false);
    };

    let cancel = CancellationToken::new();
    let run_id = match super::subagent_registry::register(
        &params.parent_session_id,
        &params.child_session_id,
        cancel.clone(),
    )
    .await
    {
        Ok(run_id) => run_id,
        Err(e) => {
            let _ = super::session_subagents::mark_status(
                &params.child_session_id,
                super::subagent_status::FAILED,
            )
            .await;
            return Err(e);
        }
    };

    let preview = prompt.chars().take(MAX_PROMPT_PREVIEW).collect();
    let _ = params.parent_emitter.send(StreamEvent::SubagentSpawned {
        subagent_session_id: params.child_session_id.clone(),
        subagent_name: name,
        subagent_type: params.subagent_type.clone(),
        subagent_description: description,
        subagent_color_key: color_key,
        prompt_preview: preview,
        run_id: Some(run_id),
    });

    if let Err(e) =
        super::subagent_spawn_channel::send(super::subagent_spawn_channel::SpawnRequest {
            app: params.app,
            parent_session_id: params.parent_session_id,
            child_session_id: params.child_session_id.clone(),
            model: params.model,
            provider: params.provider,
            prompt,
            subagent_type: params.subagent_type,
            parent_emitter: params.parent_emitter,
            cancel,
            project_id: params.project_id,
            detached: true,
            completion_tx: None,
        })
    {
        super::subagent_registry::unregister(&params.child_session_id).await;
        let _ = super::session_subagents::mark_status(
            &params.child_session_id,
            super::subagent_status::FAILED,
        )
        .await;
        return Err(e);
    }

    Ok(true)
}

async fn take_next_prompt(
    child_id: &str,
) -> Result<Option<(String, String, String, String)>, String> {
    let mut child = super::session_store::get(child_id).await?;
    if child.subagent_queued_prompts.is_empty() {
        return Ok(None);
    }
    let prompt = child.subagent_queued_prompts.remove(0);
    child.subagent_prompt = Some(prompt.clone());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.subagent_summary = None;
    child.updated_at = Some(chrono::Utc::now());
    let name = child.name.clone();
    let description = child.subagent_description.clone().unwrap_or_default();
    let color_key = child.subagent_color_key.clone().unwrap_or_else(|| {
        super::subagent_profile::default_color_key(
            child.subagent_type.as_deref().unwrap_or("explorer"),
        )
        .to_string()
    });
    let user_msg = AgentMessage {
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
    child.messages.push(user_msg);
    super::session_store::save(&child).await?;
    Ok(Some((prompt, name, description, color_key)))
}
