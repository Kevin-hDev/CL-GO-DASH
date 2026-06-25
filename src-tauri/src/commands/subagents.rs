use super::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use super::subagents_validation::validate_session_id;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::types_session::AgentSessionMeta;
use crate::services::agent_local::{session_store, session_subagents, subagent_registry};
use crate::ActiveStreams;
use serde::Serialize;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

#[cfg(test)]
pub use super::subagents_validation::validate_session_id_for_test;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubagentInfo {
    pub session_id: String,
    pub name: String,
    pub subagent_type: String,
    pub status: String,
    pub prompt_preview: String,
    pub run_id: Option<String>,
}

#[tauri::command]
pub async fn list_subagents(
    parent_session_id: String,
    run_id: Option<String>,
) -> Result<Vec<AgentSessionMeta>, String> {
    validate_session_id(&parent_session_id)?;
    let all = session_store::list().await?;
    Ok(all
        .into_iter()
        .filter(|s| {
            s.parent_session_id.as_deref() == Some(&parent_session_id)
                && run_id
                    .as_ref()
                    .is_none_or(|rid| s.subagent_run_id.as_deref() == Some(rid))
        })
        .collect())
}

#[tauri::command]
pub async fn get_active_subagents(parent_session_id: String) -> Result<Vec<SubagentInfo>, String> {
    validate_session_id(&parent_session_id)?;
    let entries = subagent_registry::list_for_parent(&parent_session_id).await;
    Ok(entries
        .into_iter()
        .map(|e| SubagentInfo {
            session_id: e.session_id,
            name: e.name,
            subagent_type: e.subagent_type,
            status: "running".to_string(),
            prompt_preview: String::new(),
            run_id: Some(e.run_id),
        })
        .collect())
}

#[tauri::command]
pub async fn cancel_subagent(subagent_session_id: String) -> Result<(), String> {
    validate_session_id(&subagent_session_id)?;
    if subagent_registry::cancel_one(&subagent_session_id).await {
        let _ = session_subagents::mark_status(&subagent_session_id, "cancelled").await;
        Ok(())
    } else {
        Err("Sous-agent introuvable ou déjà terminé".to_string())
    }
}

#[tauri::command]
pub async fn synthesize_subagent_results(
    app: tauri::AppHandle,
    parent_session_id: String,
    run_id: Option<String>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<u64, String> {
    validate_session_id(&parent_session_id)?;
    let parent = session_store::get(&parent_session_id).await?;
    let run = run_id.or_else(|| parent.messages.last().map(|m| m.id.clone()));
    let active = subagent_registry::list_for_parent(&parent_session_id).await;
    if !active.is_empty() {
        return Err("Des sous-agents sont encore actifs".to_string());
    }
    let children = list_subagents(parent_session_id.clone(), run.clone()).await?;
    if children.is_empty() {
        return Err("Aucun résultat de sous-agent à synthétiser".to_string());
    }

    let synthesis_prompt = "Tous les sous-agents ont terminé. Synthétise leurs résultats ci-dessus en une réponse structurée et concise.".to_string();
    let user_msg = crate::services::agent_local::types_session::AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: synthesis_prompt,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: 0,
        skill_names: None,
    };
    session_store::add_messages(&parent_session_id, vec![user_msg], 0).await?;
    let fresh = session_store::get(&parent_session_id).await?;
    let messages = fresh
        .messages
        .iter()
        .map(to_chat_message)
        .collect::<Vec<_>>();

    let cancel = CancellationToken::new();
    let generation = crate::STREAM_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let old_stream = {
        let mut map = streams.0.lock().await;
        map.remove(&parent_session_id)
    };
    if let Some((old_token, _, old_request_id)) = old_stream {
        old_token.cancel();
        crate::services::agent_local::stream_diagnostics::record_cancelled(
            &parent_session_id,
            &old_request_id,
        )
        .await;
    }
    let request_id = crate::services::agent_local::stream_diagnostics::start_request(
        &parent_session_id,
        generation,
    )
    .await;
    {
        let mut map = streams.0.lock().await;
        map.insert(
            parent_session_id.clone(),
            (cancel.clone(), generation, request_id.clone()),
        );
    }

    let task_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let emitter = AgentEventEmitter::new(task_app.clone(), parent_session_id.clone());
        let _ = emitter.send(StreamEvent::SessionSnapshot {
            messages: fresh.messages.clone(),
            token_count: fresh.accumulated_tokens,
        });
        let result = run_stream_task(StreamTaskParams {
            on_event: emitter.clone(),
            session_id: parent_session_id.clone(),
            request_id: request_id.clone(),
            model: fresh.model,
            messages,
            tools: vec![],
            think: false,
            provider: fresh.provider,
            working_dir: None,
            capability_hints: StreamCapabilityHints::default(),
            reasoning_mode: None,
            permission_mode_override: Some("auto".to_string()),
            cancel,
        })
        .await;

        let is_current = {
            let state = task_app.state::<ActiveStreams>();
            let mut map = state.0.lock().await;
            match map.get(&parent_session_id) {
                Some((_, gen, _)) if *gen == generation => {
                    map.remove(&parent_session_id);
                    true
                }
                _ => false,
            }
        };
        if let Err(message) = result {
            if is_current && message != "Annulé" {
                let diagnostic = crate::services::agent_local::stream_diagnostics::record_failure(
                    &parent_session_id,
                    Some(&request_id),
                    &message,
                    false,
                )
                .await;
                let _ = emitter.send(StreamEvent::Error {
                    message,
                    is_connection: false,
                    diagnostic,
                });
            }
        }
    });

    Ok(generation)
}

fn to_chat_message(
    message: &crate::services::agent_local::types_session::AgentMessage,
) -> ChatMessage {
    ChatMessage {
        role: message.role.clone(),
        content: message.content.clone(),
        tool_calls: message.tool_calls.as_ref().map(|calls| {
            calls
                .iter()
                .map(
                    |call| crate::services::agent_local::types_ollama::ToolCallOllama {
                        id: None,
                        extra_content: call.extra_content.clone(),
                        function: crate::services::agent_local::types_ollama::ToolCallFunction {
                            name: call.function.name.clone(),
                            arguments: call.function.arguments.clone(),
                        },
                    },
                )
                .collect()
        }),
        tool_name: message.tool_name.clone(),
        reasoning_content: message.thinking.clone(),
        ..Default::default()
    }
}
