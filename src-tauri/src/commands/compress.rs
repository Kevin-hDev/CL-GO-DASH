//! Commande Tauri pour la compression manuelle de contexte.
//!
//! Lit la session, lance la compression via le LLM actif, met à jour
//! les messages de la session et émet les événements Compressing.

use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::types_session::AgentMessage;
use crate::services::compress::{engine, prompt};
use chrono::Utc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Construit une liste de `ChatMessage` depuis les `AgentMessage` d'une session.
/// Exclut les messages système, les messages sans contenu textuel et les résultats d'outils.
fn agent_messages_to_chat(messages: &[AgentMessage]) -> Vec<ChatMessage> {
    messages
        .iter()
        .filter(|m| matches!(m.role.as_str(), "user" | "assistant"))
        .filter(|m| !m.content.is_empty())
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        })
        .collect()
}

/// Crée les `AgentMessage` représentant l'état post-compression.
/// Boundary marker + résumé (role user avec contenu formaté).
fn build_compressed_agent_messages(summary: &str) -> Vec<AgentMessage> {
    let boundary = AgentMessage {
        id: Uuid::new_v4().to_string(),
        role: "system".to_string(),
        content: engine::BOUNDARY_CONTENT.to_string(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        skill_names: None,
    };
    let summary_msg = AgentMessage {
        id: Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: prompt::format_summary_message(summary, false),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        skill_names: None,
    };
    vec![boundary, summary_msg]
}

async fn run_compression(
    on_event: &AgentEventEmitter,
    session_id: &str,
    provider: &str,
    model: &str,
) -> Result<(), String> {
    let _ = on_event.send(StreamEvent::Compressing { status: "start".to_string() });

    let session = session_store::get(session_id).await?;

    if session.messages.len() < 4 {
        let _ = on_event.send(StreamEvent::Compressing { status: "done".to_string() });
        return Err("Pas assez de messages à compresser".to_string());
    }

    let chat_msgs = agent_messages_to_chat(&session.messages);
    if chat_msgs.len() < 2 {
        let _ = on_event.send(StreamEvent::Compressing { status: "done".to_string() });
        return Err("Pas assez de messages éligibles".to_string());
    }

    let compress_request = engine::build_compression_request_content(&chat_msgs, None);
    let cancel = CancellationToken::new();

    let summary_raw = if provider == "ollama" {
        crate::services::agent_local::ollama_stream::collect_chat(model, compress_request)
            .await
            .map(|(content, _)| content)
            .map_err(|e| format!("Compression Ollama : {e}"))?
    } else {
        crate::services::llm::stream::collect_chat_silent(provider, model, &compress_request, cancel)
            .await
            .map(|r| r.content)
            .map_err(|e| format!("Compression LLM : {e}"))?
    };

    let summary = prompt::extract_summary(&summary_raw);

    let compressed_messages = build_compressed_agent_messages(&summary);

    let lock = session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut updated_session = session_store::get(session_id).await?;
    updated_session.messages = compressed_messages;
    updated_session.accumulated_tokens = 0;
    session_store::save(&updated_session).await?;

    let _ = on_event.send(StreamEvent::Compressing { status: "done".to_string() });
    Ok(())
}

#[tauri::command]
pub async fn compress_conversation(
    app: tauri::AppHandle,
    session_id: String,
    provider: String,
    model: String,
) -> Result<(), String> {
    session_store::validate_session_id(&session_id)
        .map_err(|_| "Identifiant de session invalide".to_string())?;

    let emitter = AgentEventEmitter::new(app.clone(), session_id.clone());

    let session_id_clone = session_id.clone();
    let provider_clone = provider.clone();
    let model_clone = model.clone();
    let emitter_clone = emitter.clone();

    tauri::async_runtime::spawn(async move {
        let result = run_compression(&emitter_clone, &session_id_clone, &provider_clone, &model_clone).await;
        if let Err(e) = result {
            eprintln!("[compress_manual] Échec : {e}");
            let _ = emitter_clone.send(StreamEvent::Compressing { status: "done".to_string() });
        }
    });

    Ok(())
}
