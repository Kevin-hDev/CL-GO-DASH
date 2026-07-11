use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) async fn run_inner(
    app: AppHandle,
    child_session_id: String,
    model: String,
    provider: String,
    prompt: String,
    subagent_type: String,
    cancel: CancellationToken,
    project_id: Option<String>,
    working_dir: String,
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
    let emitter = AgentEventEmitter::new(app, child_session_id.clone());
    let request_id = super::stream_diagnostics::start_request(&child_session_id, 0).await;
    super::subagent_activity::record_status(&child_session_id, "Démarré", None).await;
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
        working_dir: Some(working_dir),
        capability_hints: StreamCapabilityHints::default(),
        reasoning_mode: None,
        permission_mode_override: Some("subagent".to_string()),
        plan_mode: Some(false),
        cancel: cancel.clone(),
    })
    .await;

    finalize_stream_result(result, &child_session_id, &request_id, cancel).await
}

async fn finalize_stream_result(
    result: Result<Vec<super::types_ollama::ChatMessage>, String>,
    child_session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
) -> Result<(bool, String, String), String> {
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
        Err(e) if super::subagent_instruction_delivery::is_delivery_error(&e) => Err(e),
        Err(_) => {
            super::stream_diagnostics::record_failure(
                child_session_id,
                Some(request_id),
                "Le sous-agent n'a pas pu terminer correctement.",
                false,
            )
            .await;
            Err("Le sous-agent n'a pas pu terminer correctement.".to_string())
        }
    }
}
