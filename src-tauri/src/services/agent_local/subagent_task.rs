use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_registry;
use crate::services::agent_local::types_ollama::StreamEvent;
use serde_json::json;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

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
    let run_id = subagent_registry::get_run_id_for_child(&child_session_id).await;
    super::subagent_flow_log::record(
        "task_run_started",
        Some(&parent_session_id),
        Some(&child_session_id),
        run_id.as_deref(),
        json!({"type": subagent_type}),
    );
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
        Err(e) => {
            eprintln!("[subagent] échec {}: {e}", child_session_id);
            (
                false,
                super::subagent_status::FAILED.to_string(),
                "Le sous-agent n'a pas pu terminer correctement.".to_string(),
            )
        }
    };
    super::subagent_flow_log::record(
        "task_run_finished",
        Some(&parent_session_id),
        Some(&child_session_id),
        run_id.as_deref(),
        json!({"status": status.as_str(), "success": success}),
    );
    let queued_followup = has_queued_followup(&child_session_id, &status).await;
    let session_status = effective_session_status(&status, queued_followup);

    if let Err(e) = update_session_status(&child_session_id, session_status).await {
        // Non fatal : on logge mais on continue. Le statut disque sera
        // reclassé en "interrupted" au prochain démarrage par le cleanup.
        eprintln!("[subagent] persistance statut {}: {e}", child_session_id);
    }
    if let Err(e) = update_session_summary(&child_session_id, &summary, session_status).await {
        eprintln!("[subagent] persistance résumé {}: {e}", child_session_id);
    }
    super::subagent_flow_log::record(
        "session_final_state_persisted",
        Some(&parent_session_id),
        Some(&child_session_id),
        run_id.as_deref(),
        json!({"status": session_status, "queued_followup": queued_followup}),
    );

    let child_name = get_child_name(&child_session_id).await;
    let report = super::subagent_hidden_reports::build_report(
        child_session_id.clone(),
        child_name.clone(),
        subagent_type.clone(),
        session_status.to_string(),
        summary.clone(),
    );
    if let Err(e) = super::subagent_hidden_reports::append(&parent_session_id, report).await {
        eprintln!("[subagent] rapport parent {}: {e}", parent_session_id);
        super::subagent_flow_log::record(
            "hidden_report_append_failed",
            Some(&parent_session_id),
            Some(&child_session_id),
            run_id.as_deref(),
            json!({}),
        );
    } else {
        super::subagent_flow_log::record(
            "hidden_report_appended",
            Some(&parent_session_id),
            Some(&child_session_id),
            run_id.as_deref(),
            json!({"status": session_status}),
        );
    }

    if queued_followup {
        super::subagent_flow_log::record(
            "completion_event_skipped_for_queue",
            Some(&parent_session_id),
            Some(&child_session_id),
            run_id.as_deref(),
            json!({"status": status.as_str()}),
        );
    } else {
        let _ = parent_emitter.send(StreamEvent::SubagentCompleted {
            subagent_session_id: child_session_id.clone(),
            success,
            status: status.clone(),
            summary: summary.clone(),
            run_id,
        });
    }
    let current_run_id = subagent_registry::get_run_id_for_child(&child_session_id).await;
    super::subagent_flow_log::record(
        "completion_event_sent",
        Some(&parent_session_id),
        Some(&child_session_id),
        current_run_id.as_deref(),
        json!({"status": session_status, "success": success, "sent": !queued_followup}),
    );

    super::subagent_working_dir::cleanup(&child_session_id).await;
    subagent_registry::unregister(&child_session_id).await;
    super::subagent_flow_log::record(
        "registry_unregistered",
        Some(&parent_session_id),
        Some(&child_session_id),
        None,
        json!({}),
    );

    if let Err(e) = super::subagent_queued::spawn_next_if_present(next_run).await {
        eprintln!("[subagent] relance file {}: {e}", child_session_id);
    }
}

async fn get_child_name(child_id: &str) -> String {
    session_store::get(child_id)
        .await
        .map(|s| s.name.clone())
        .unwrap_or_else(|_| "agent".to_string())
}

async fn has_queued_followup(child_id: &str, status: &str) -> bool {
    status == super::subagent_status::COMPLETED
        && session_store::get(child_id)
            .await
            .map(|session| !session.subagent_queued_prompts.is_empty())
            .unwrap_or(false)
}

pub fn effective_session_status(status: &str, queued_followup: bool) -> &str {
    if queued_followup {
        super::subagent_status::RUNNING
    } else {
        status
    }
}

async fn update_session_status(session_id: &str, status: &str) -> Result<(), String> {
    super::session_subagents::mark_status(session_id, status).await
}

async fn update_session_summary(
    session_id: &str,
    summary: &str,
    status: &str,
) -> Result<(), String> {
    let mut session = session_store::get(session_id).await?;
    session.subagent_summary = Some(summary.to_string());
    session.subagent_last_activity = Some(super::types_session::SubagentLastActivity {
        kind: "status".to_string(),
        label: final_activity_label(status).to_string(),
        detail: Some(summary.chars().take(220).collect()),
        updated_at: chrono::Utc::now(),
    });
    session_store::save(&session).await
}

fn final_activity_label(status: &str) -> &'static str {
    match status {
        super::subagent_status::CANCELLED => "Annulé",
        super::subagent_status::FAILED => "Échoué",
        _ => "Terminé",
    }
}
