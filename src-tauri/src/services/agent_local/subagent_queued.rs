use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) use super::subagent_queued_transition::{
    finalize_unstarted_followup, prepare_next_followup,
};

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

pub(super) struct PreparedFollowup {
    pub cancel: CancellationToken,
    pub color_key: String,
    pub description: String,
    pub name: String,
    pub prompt: String,
    pub run_id: String,
}

pub async fn spawn_next_if_present(params: QueuedSubagentRun) -> Result<bool, String> {
    let Some(prepared) = prepare_next_followup(
        &params.parent_session_id,
        &params.child_session_id,
        &params.subagent_type,
    )
    .await?
    else {
        return Ok(false);
    };

    let preview = prepared.prompt.chars().take(MAX_PROMPT_PREVIEW).collect();
    let _ = params.parent_emitter.send(StreamEvent::SubagentSpawned {
        subagent_session_id: params.child_session_id.clone(),
        subagent_name: prepared.name,
        subagent_type: params.subagent_type.clone(),
        subagent_description: prepared.description,
        subagent_color_key: prepared.color_key,
        prompt_preview: preview,
        run_id: Some(prepared.run_id),
    });

    let failure_parent_id = params.parent_session_id.clone();
    let failure_child_id = params.child_session_id.clone();
    let failure_subagent_type = params.subagent_type.clone();
    let send_result =
        super::subagent_spawn_channel::send(super::subagent_spawn_channel::SpawnRequest {
            app: params.app,
            parent_session_id: params.parent_session_id,
            child_session_id: params.child_session_id.clone(),
            model: params.model,
            provider: params.provider,
            prompt: prepared.prompt,
            subagent_type: params.subagent_type,
            parent_emitter: params.parent_emitter,
            cancel: prepared.cancel,
            project_id: params.project_id,
        });
    finalize_spawn_send_result(
        send_result,
        &failure_parent_id,
        &failure_child_id,
        &failure_subagent_type,
    )
    .await?;

    Ok(true)
}

pub(super) async fn finalize_spawn_send_result(
    send_result: Result<(), String>,
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
) -> Result<(), String> {
    let Err(original_error) = send_result else {
        return Ok(());
    };
    match finalize_unstarted_followup(parent_session_id, child_session_id, subagent_type).await {
        Ok(()) => Err(original_error),
        Err(error) => Err(error),
    }
}
