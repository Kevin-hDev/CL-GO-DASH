use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_registry;
use crate::services::agent_local::types_ollama::StreamEvent;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

struct FinalizedSubagent {
    queued_followup: bool,
    session_status: String,
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
    let next_run = super::subagent_queued::QueuedSubagentRun {
        app: app.clone(),
        parent_session_id: parent_session_id.clone(),
        child_session_id: child_session_id.clone(),
        model: model.clone(),
        provider: provider.clone(),
        subagent_type: subagent_type.clone(),
        parent_emitter: parent_emitter.clone(),
        project_id: project_id.clone(),
    };
    let result = super::subagent_task_stream::run_inner(
        app,
        child_session_id.clone(),
        model,
        provider,
        prompt,
        subagent_type.clone(),
        cancel,
        project_id,
    )
    .await;

    let run_id = subagent_registry::get_run_id_for_child(&child_session_id).await;

    let (success, status, summary) = match result {
        Ok(s) => s,
        Err(_) => {
            eprintln!("[subagent] échec {}", child_session_id);
            (
                false,
                super::subagent_status::FAILED.to_string(),
                "Le sous-agent n'a pas pu terminer correctement.".to_string(),
            )
        }
    };
    let finalized = match finalize_session_after_run(&child_session_id, &status, &summary).await {
        Ok(value) => value,
        Err(_) => {
            eprintln!("[subagent] persistance statut {}", child_session_id);
            subagent_registry::unregister(&child_session_id).await;
            FinalizedSubagent {
                queued_followup: false,
                session_status: status.clone(),
            }
        }
    };

    let child_name = get_child_name(&child_session_id).await;
    let report = super::subagent_hidden_reports::build_report(
        child_session_id.clone(),
        child_name.clone(),
        subagent_type.clone(),
        finalized.session_status.clone(),
        summary.clone(),
    );
    if super::subagent_hidden_reports::append(&parent_session_id, report)
        .await
        .is_err()
    {
        eprintln!("[subagent] rapport parent {}", parent_session_id);
    }

    if !finalized.queued_followup {
        let _ = parent_emitter.send(StreamEvent::SubagentCompleted {
            subagent_session_id: child_session_id.clone(),
            success,
            status: status.clone(),
            summary: summary.clone(),
            run_id,
        });
    }

    super::subagent_working_dir::cleanup(&child_session_id).await;

    if finalized.queued_followup {
        subagent_registry::unregister(&child_session_id).await;
    }

    if finalized.queued_followup
        && super::subagent_queued::spawn_next_if_present(next_run)
            .await
            .is_err()
    {
        eprintln!("[subagent] relance file {}", child_session_id);
    }
}

async fn get_child_name(child_id: &str) -> String {
    session_store::get(child_id)
        .await
        .map(|s| s.name.clone())
        .unwrap_or_else(|_| "agent".to_string())
}

pub fn effective_session_status(status: &str, queued_followup: bool) -> &str {
    if queued_followup {
        super::subagent_status::RUNNING
    } else {
        status
    }
}

async fn finalize_session_after_run(
    session_id: &str,
    status: &str,
    summary: &str,
) -> Result<FinalizedSubagent, String> {
    let lock = session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = session_store::get(session_id).await?;
    let queued_followup =
        status == super::subagent_status::COMPLETED && !session.subagent_queued_prompts.is_empty();
    let session_status = effective_session_status(status, queued_followup);
    if !queued_followup {
        session.subagent_queued_prompts.clear();
    }
    session.subagent_summary = Some(summary.to_string());
    session.subagent_status = Some(session_status.to_string());
    session.subagent_last_activity = Some(super::types_session::SubagentLastActivity {
        kind: "status".to_string(),
        label: final_activity_label(session_status).to_string(),
        detail: Some(summary.chars().take(220).collect()),
        updated_at: chrono::Utc::now(),
    });
    session.updated_at = Some(chrono::Utc::now());
    let save_result = session_store::save(&session).await;
    if !queued_followup {
        subagent_registry::unregister(session_id).await;
    }
    save_result.map(|_| FinalizedSubagent {
        queued_followup,
        session_status: session_status.to_string(),
    })
}

pub(super) fn final_activity_label(status: &str) -> &'static str {
    match status {
        super::subagent_status::RUNNING => "En cours",
        super::subagent_status::CANCELLED => "Annulé",
        super::subagent_status::FAILED => "Échoué",
        _ => "Terminé",
    }
}
