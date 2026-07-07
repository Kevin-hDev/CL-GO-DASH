use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_completion::SubagentCompletion;
use crate::services::agent_local::subagent_registry;
use crate::services::agent_local::types_ollama::StreamEvent;
use tauri::AppHandle;
use tokio::sync::oneshot;
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
    detached: bool,
    completion_tx: Option<oneshot::Sender<SubagentCompletion>>,
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
    let result = run_inner(
        app,
        child_session_id.clone(),
        model,
        provider,
        prompt,
        subagent_type.clone(),
        &parent_emitter,
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

    if let Err(e) = update_session_status(&child_session_id, &status).await {
        // Non fatal : on logge mais on continue. Le statut disque sera
        // reclassé en "interrupted" au prochain démarrage par le cleanup.
        eprintln!("[subagent] persistance statut {}: {e}", child_session_id);
    }
    if let Err(e) = update_session_summary(&child_session_id, &summary).await {
        eprintln!("[subagent] persistance résumé {}: {e}", child_session_id);
    }

    let child_name = get_child_name(&child_session_id).await;

    super::subagent_working_dir::cleanup(&child_session_id).await;
    subagent_registry::unregister(&child_session_id).await;

    let completion = SubagentCompletion {
        child_session_id: child_session_id.clone(),
        name: child_name,
        subagent_type,
        status: status.clone(),
        success,
        summary: summary.clone(),
        run_id: run_id.clone(),
    };
    let _ = parent_emitter.send(StreamEvent::SubagentCompleted {
        subagent_session_id: child_session_id,
        success,
        status,
        summary,
        run_id,
    });
    if detached {
        let report = super::subagent_hidden_reports::build_report(
            completion.child_session_id.clone(),
            completion.name.clone(),
            completion.subagent_type.clone(),
            completion.status.clone(),
            completion.summary.clone(),
        );
        if let Err(e) = super::subagent_hidden_reports::append(&parent_session_id, report).await {
            eprintln!("[subagent] rapport parent {}: {e}", parent_session_id);
        }
        if let Err(e) = super::subagent_queued::spawn_next_if_present(next_run).await {
            eprintln!(
                "[subagent] relance file {}: {e}",
                completion.child_session_id
            );
        }
    }
    if let Some(tx) = completion_tx {
        let _ = tx.send(completion);
    }
}

async fn get_child_name(child_id: &str) -> String {
    session_store::get(child_id)
        .await
        .map(|s| s.name.clone())
        .unwrap_or_else(|_| "agent".to_string())
}

async fn run_inner(
    app: AppHandle,
    child_session_id: String,
    model: String,
    provider: String,
    prompt: String,
    subagent_type: String,
    _parent_emitter: &AgentEventEmitter,
    cancel: CancellationToken,
    project_id: Option<String>,
) -> Result<(bool, String, String), String> {
    let is_explorer = subagent_type == "explorer";
    let tools = if is_explorer {
        super::tool_definitions_subagent::get_explorer_tool_definitions()
    } else {
        super::tool_dispatcher::get_tool_definitions()
    };

    let system_prompt = if is_explorer {
        super::subagent_prompts::explorer_system()
    } else {
        super::subagent_prompts::coder_system(project_id.as_deref()).await
    };

    let messages =
        super::subagent_context::build_messages(&child_session_id, system_prompt, &prompt).await;

    let working_dir =
        super::subagent_working_dir::resolve(project_id.as_deref(), &child_session_id, is_explorer)
            .await?;
    let emitter = AgentEventEmitter::new(app, child_session_id.clone());
    let request_id = super::stream_diagnostics::start_request(&child_session_id, 0).await;

    if let Ok(child_session) = session_store::get(&child_session_id).await {
        let _ = emitter.send(StreamEvent::SessionSnapshot {
            messages: child_session.messages,
            token_count: child_session.accumulated_tokens,
        });
    }

    let result = run_stream_task(StreamTaskParams {
        on_event: emitter,
        session_id: child_session_id.clone(),
        request_id: request_id.clone(),
        model,
        messages,
        tools,
        think: false,
        provider,
        working_dir: Some(working_dir.to_string_lossy().to_string()),
        capability_hints: StreamCapabilityHints::default(),
        reasoning_mode: None,
        permission_mode_override: Some("subagent".to_string()),
        plan_mode: Some(false),
        cancel: cancel.clone(),
    })
    .await;

    let was_cancelled = cancel.is_cancelled();
    match result {
        Ok(final_msgs) => {
            let summary = super::subagent_summary::extract_summary_from_messages(&final_msgs);
            let status = if was_cancelled {
                super::subagent_status::CANCELLED
            } else {
                super::subagent_status::COMPLETED
            };
            Ok((!was_cancelled, status.to_string(), summary))
        }
        Err(e) if was_cancelled || e == "Annulé" => Ok((
            false,
            super::subagent_status::CANCELLED.to_string(),
            "Sous-agent annulé.".to_string(),
        )),
        Err(e) => {
            super::stream_diagnostics::record_failure(
                &child_session_id,
                Some(&request_id),
                &e,
                false,
            )
            .await;
            Err(e)
        }
    }
}

async fn update_session_status(session_id: &str, status: &str) -> Result<(), String> {
    super::session_subagents::mark_status(session_id, status).await
}

async fn update_session_summary(session_id: &str, summary: &str) -> Result<(), String> {
    let mut session = session_store::get(session_id).await?;
    session.subagent_summary = Some(summary.to_string());
    session_store::save(&session).await
}
