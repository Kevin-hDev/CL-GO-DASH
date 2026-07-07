use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_registry;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub async fn run(
    app: AppHandle,
    child_session_id: String,
    model: String,
    provider: String,
    prompt: String,
    subagent_type: String,
    parent_emitter: AgentEventEmitter,
    cancel: CancellationToken,
    project_id: Option<String>,
) {
    let result = run_inner(
        app,
        child_session_id.clone(),
        model,
        provider,
        prompt,
        subagent_type,
        &parent_emitter,
        cancel,
        project_id,
    )
    .await;

    let parent_session_id = parent_emitter.session_id().to_string();
    let run_id = subagent_registry::get_run_id_for_child(&child_session_id).await;

    let (mut success, mut status, mut summary) = match result {
        Ok(s) => s,
        Err(e) => (
            false,
            super::subagent_status::FAILED.to_string(),
            format!("Erreur : {e}"),
        ),
    };

    if let Err(e) = update_session_status(&child_session_id, &status).await {
        // Non fatal : on logge mais on continue. Le statut disque sera
        // reclassé en "interrupted" au prochain démarrage par le cleanup.
        eprintln!("[subagent] persistance statut {}: {e}", child_session_id);
    }

    let child_name = super::subagent_orchestrator::get_child_name(&child_session_id).await;
    if let Err(e) = super::subagent_orchestrator::inject_summary_in_parent(
        &parent_session_id,
        &child_session_id,
        &child_name,
        &summary,
        success,
    )
    .await
    {
        // Fail closed : on rend l'échec visible à l'utilisateur via le
        // résumé remonté au parent, plutôt que d'avaler silencieusement.
        eprintln!(
            "[subagent] injection rapport parent {}: {e}",
            child_session_id
        );
        success = false;
        status = super::subagent_status::FAILED.to_string();
        if let Err(mark_err) = update_session_status(&child_session_id, &status).await {
            eprintln!(
                "[subagent] persistance statut failed après injection {}: {mark_err}",
                child_session_id
            );
        }
        summary = format!("{summary}\n\nRapport non injecté dans la session parente.");
    }

    super::subagent_working_dir::cleanup(&child_session_id).await;
    subagent_registry::unregister(&child_session_id).await;

    let remaining = subagent_registry::list_for_parent(&parent_session_id).await;
    let _ = parent_emitter.send(StreamEvent::SubagentCompleted {
        subagent_session_id: child_session_id,
        success,
        status,
        summary,
        all_done: remaining.is_empty(),
        run_id,
    });
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

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
            ..Default::default()
        },
        ChatMessage {
            role: "user".to_string(),
            content: prompt.clone(),
            ..Default::default()
        },
    ];

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
