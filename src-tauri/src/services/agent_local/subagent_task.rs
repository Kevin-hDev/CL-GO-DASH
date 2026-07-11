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
) {
    loop {
        let result = super::subagent_task_stream::run_inner(
            app.clone(),
            child_session_id.clone(),
            model.clone(),
            provider.clone(),
            prompt.clone(),
            subagent_type.clone(),
            cancel.clone(),
            project_id.clone(),
        )
        .await;
        let run_id = subagent_registry::get_run_id_for_child(&child_session_id).await;
        let (mut success, mut status, mut summary) = match result {
            Ok(value) => value,
            Err(error) if super::subagent_instruction_delivery::is_delivery_error(&error) => {
                let success = false;
                let status = super::subagent_status::FAILED.to_string();
                let summary = super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string();
                let _ = super::subagent_completion::persist_instruction_delivery_failure(
                    &parent_session_id,
                    &child_session_id,
                    &subagent_type,
                )
                .await;
                let _ = parent_emitter.send(StreamEvent::SubagentCompleted {
                    subagent_session_id: child_session_id.clone(),
                    success,
                    status,
                    summary,
                    run_id,
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
        let finalized = match super::subagent_completion::persist_terminal_completion(
            &parent_session_id,
            &child_session_id,
            &subagent_type,
            &status,
            &summary,
        )
        .await
        {
            Ok(value) => value,
            Err(_) => {
                success = false;
                status = super::subagent_status::FAILED.to_string();
                summary = super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string();
                let _ = parent_emitter.send(StreamEvent::SubagentCompleted {
                    subagent_session_id: child_session_id.clone(),
                    success,
                    status,
                    summary,
                    run_id,
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
            run_id,
        });
        break;
    }

    super::subagent_working_dir::cleanup(&child_session_id).await;
    session_store::remove_session_lock(&child_session_id).await;
}

pub fn effective_session_status(status: &str, queued_followup: bool) -> &str {
    if queued_followup {
        super::subagent_status::RUNNING
    } else {
        status
    }
}

pub(super) fn should_continue_same_run(status: &str, has_queued_prompt: bool) -> bool {
    status == super::subagent_status::COMPLETED && has_queued_prompt
}

pub(super) fn finalize_loaded_session_after_run(
    session: &mut AgentSession,
    status: &str,
    summary: &str,
) -> FinalizedSubagent {
    let finalized = apply_finalized_subagent_state(session, status, summary);
    session.subagent_last_activity = Some(super::types_session::SubagentLastActivity {
        kind: "status".to_string(),
        label: final_activity_label(&finalized.session_status).to_string(),
        detail: Some(summary.chars().take(220).collect()),
        updated_at: chrono::Utc::now(),
    });
    finalized
}

pub(super) fn apply_finalized_subagent_state(
    session: &mut AgentSession,
    status: &str,
    summary: &str,
) -> FinalizedSubagent {
    let queued_followup =
        should_continue_same_run(status, !session.subagent_queued_prompts.is_empty());
    let session_status = effective_session_status(status, queued_followup);
    if !queued_followup {
        session.subagent_queued_prompts.clear();
    }
    session.subagent_summary = Some(summary.to_string());
    session.subagent_status = Some(session_status.to_string());
    session.updated_at = Some(chrono::Utc::now());
    FinalizedSubagent {
        queued_followup,
        session_status: session_status.to_string(),
    }
}

pub(super) fn final_activity_label(status: &str) -> &'static str {
    match status {
        super::subagent_status::RUNNING => "En cours",
        super::subagent_status::CANCELLED => "Annulé",
        super::subagent_status::FAILED => "Échoué",
        _ => "Terminé",
    }
}
