use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_registry;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::agent_local::types_session::AgentSession;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) struct FinalizedSubagent {
    pub(super) queued_followup: bool,
    pub(super) session_status: String,
}

pub async fn run(
    app: AppHandle,
    parent_session_id: String,
    child_session_id: String,
    model: String,
    provider: String,
    prompt: String,
    subagent_type: String,
    parent_emitter: AgentEventEmitter,
    cancel: CancellationToken,
    project_id: Option<String>,
    run_id: String,
    execution_id: String,
) {
    if !subagent_registry::owns_execution(&child_session_id, &run_id, &execution_id).await {
        return;
    }
    let is_explorer = subagent_type == "explorer";
    let prepared = match super::subagent_working_dir::resolve(
        project_id.as_deref(),
        &child_session_id,
        is_explorer,
        &run_id,
        &execution_id,
    )
    .await
    {
        Ok(prepared) => prepared,
        Err(_) => {
            let reported = finish_preparation_failure(
                &parent_session_id,
                &child_session_id,
                &subagent_type,
                &run_id,
                &execution_id,
            )
            .await;
            if reported {
                emit_failure(&parent_emitter, &child_session_id, &run_id);
            }
            session_store::remove_session_lock(&child_session_id).await;
            return;
        }
    };
    let working_dir = prepared.path().to_string_lossy().to_string();
    loop {
        if !subagent_registry::owns_execution(&child_session_id, &run_id, &execution_id).await {
            break;
        }
        let result = super::subagent_task_stream::run_inner(
            app.clone(),
            child_session_id.clone(),
            model.clone(),
            provider.clone(),
            prompt.clone(),
            subagent_type.clone(),
            cancel.clone(),
            project_id.clone(),
            working_dir.clone(),
        )
        .await;
        let (mut success, mut status, mut summary) = match result {
            Ok(value) => value,
            Err(error) if super::subagent_instruction_delivery::is_delivery_error(&error) => {
                let success = false;
                let status = super::subagent_status::FAILED.to_string();
                let summary = super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string();
                let reported = super::subagent_completion::persist_instruction_delivery_failure_for_execution(
                    &parent_session_id,
                    &child_session_id,
                    &subagent_type,
                    &run_id,
                    &execution_id,
                )
                .await
                .unwrap_or(false);
                if !reported {
                    break;
                }
                let _ = parent_emitter.send(StreamEvent::SubagentCompleted {
                    subagent_session_id: child_session_id.clone(),
                    success,
                    status,
                    summary,
                    run_id: Some(run_id.clone()),
                });
                break;
            }
            Err(_) => {
                eprintln!("[subagent] échec {}", child_session_id);
                (
                    false,
                    super::subagent_status::FAILED.to_string(),
                    "Le sous-agent n'a pas pu terminer correctement.".to_string(),
                )
            }
        };
        let finalized = match super::subagent_completion::persist_terminal_completion_for_execution(
            &parent_session_id,
            &child_session_id,
            &subagent_type,
            &status,
            &summary,
            &run_id,
            &execution_id,
        )
        .await
        {
            Ok(Some(value)) => value,
            Ok(None) => break,
            Err(_) => {
                success = false;
                status = super::subagent_status::FAILED.to_string();
                summary = super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string();
                let _ = parent_emitter.send(StreamEvent::SubagentCompleted {
                    subagent_session_id: child_session_id.clone(),
                    success,
                    status,
                    summary,
                    run_id: Some(run_id.clone()),
                });
                break;
            }
        };
        if finalized.queued_followup {
            continue;
        }
        let _ = parent_emitter.send(StreamEvent::SubagentCompleted {
            subagent_session_id: child_session_id.clone(),
            success,
            status,
            summary,
            run_id: Some(run_id.clone()),
        });
        break;
    }

    super::subagent_working_dir::cleanup_owned(&child_session_id, prepared.worktree_path()).await;
    session_store::remove_session_lock(&child_session_id).await;
}

pub(super) async fn finish_preparation_failure(
    parent_id: &str,
    child_id: &str,
    subagent_type: &str,
    run_id: &str,
    execution_id: &str,
) -> bool {
    let summary = "Le sous-agent n'a pas pu terminer correctement.";
    !matches!(
        super::subagent_completion::persist_terminal_completion_for_execution(
        parent_id,
        child_id,
        subagent_type,
        super::subagent_status::FAILED,
        summary,
        run_id,
        execution_id,
    )
    .await,
        Ok(None)
    )
}

fn emit_failure(emitter: &AgentEventEmitter, child_id: &str, run_id: &str) {
    let summary = "Le sous-agent n'a pas pu terminer correctement.";
    let _ = emitter.send(StreamEvent::SubagentCompleted {
        subagent_session_id: child_id.to_string(),
        success: false,
        status: super::subagent_status::FAILED.to_string(),
        summary: summary.to_string(),
        run_id: Some(run_id.to_string()),
    });
}

include!("subagent_task_state.rs");
