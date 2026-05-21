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

    let (success, status, summary) = match result {
        Ok(s) => s,
        Err(e) => (false, "failed".to_string(), format!("Erreur : {e}")),
    };

    update_session_status(&child_session_id, &status).await;

    let child_name = super::subagent_orchestrator::get_child_name(&child_session_id).await;
    super::subagent_orchestrator::inject_summary_in_parent(
        &parent_session_id,
        &child_session_id,
        &child_name,
        &summary,
        success,
    )
    .await;

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

    if let Ok(child_session) = session_store::get(&child_session_id).await {
        let _ = emitter.send(StreamEvent::SessionSnapshot {
            messages: child_session.messages,
            token_count: child_session.accumulated_tokens,
        });
    }

    let result = crate::commands::agent_chat_task::run_stream_task(
        emitter,
        child_session_id.clone(),
        model,
        messages,
        tools,
        false,
        provider,
        Some(working_dir.to_string_lossy().to_string()),
        None,
        None,
        None,
        Some("subagent".to_string()),
        cancel.clone(),
    )
    .await;

    let was_cancelled = cancel.is_cancelled();
    match result {
        Ok(final_msgs) => {
            let summary = extract_summary_from_messages(&final_msgs);
            let status = if was_cancelled {
                "cancelled"
            } else {
                "completed"
            };
            Ok((!was_cancelled, status.to_string(), summary))
        }
        Err(e) if was_cancelled || e == "Annulé" => Ok((
            false,
            "cancelled".to_string(),
            "Sous-agent annulé.".to_string(),
        )),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub fn extract_summary_for_test(msgs: &[ChatMessage]) -> String {
    extract_summary_from_messages(msgs)
}

fn extract_summary_from_messages(msgs: &[ChatMessage]) -> String {
    if let Some(m) = msgs
        .iter()
        .rev()
        .find(|m| m.role == "assistant" && !m.content.trim().is_empty())
    {
        return m.content.clone();
    }
    let tool_results: Vec<&str> = msgs
        .iter()
        .rev()
        .take(6)
        .filter(|m| m.role == "tool" && !m.content.trim().is_empty())
        .map(|m| m.content.as_str())
        .collect();
    if !tool_results.is_empty() {
        let joined: String = tool_results
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n---\n");
        let truncated: String = joined.chars().take(2000).collect();
        return format!("[Résultats d'outils]\n{truncated}");
    }
    "Aucune réponse".to_string()
}

async fn update_session_status(session_id: &str, status: &str) {
    let _ = super::session_subagents::mark_status(session_id, status).await;
}
