use super::agent_chat_task::run_stream_task;

use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::ActiveStreams;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn chat_stream(
    app: tauri::AppHandle,
    session_id: String,
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<serde_json::Value>,
    think: bool,
    provider: Option<String>,
    working_dir: Option<String>,
    supports_tools: Option<bool>,
    supports_thinking: Option<bool>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<u64, String> {
    const MAX_ACTIVE_STREAMS: usize = 32;
    let cancel = CancellationToken::new();
    let generation = crate::STREAM_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    {
        let mut map = streams.0.lock().await;
        if map.len() >= MAX_ACTIVE_STREAMS {
            return Err("Trop de flux actifs simultanément".to_string());
        }
        if let Some((old_token, _)) = map.remove(&session_id) {
            old_token.cancel();
        }
        map.insert(session_id.clone(), (cancel.clone(), generation));
    }
    eprintln!("[stream] start session={session_id} gen={generation}");

    let provider = provider.unwrap_or_else(|| "ollama".to_string());
    let stream_session = session_id.clone();
    let task_app = app.clone();

    tauri::async_runtime::spawn(async move {
        let emitter = AgentEventEmitter::new(task_app.clone(), stream_session.clone());
        let result = run_stream_task(
            emitter.clone(),
            stream_session.clone(),
            model,
            messages,
            tools,
            think,
            provider,
            working_dir,
            supports_tools,
            supports_thinking,
            cancel,
        )
        .await;

        // Cleanup : ne supprime que si NOTRE génération est encore active
        // (une nouvelle requête a pu remplacer notre entrée)
        let is_current = {
            let state = task_app.state::<ActiveStreams>();
            let mut map = state.0.lock().await;
            match map.get(&stream_session) {
                Some((_, gen)) if *gen == generation => {
                    map.remove(&stream_session);
                    true
                }
                _ => false,
            }
        };

        if is_current {
            crate::services::agent_local::permission_gate::clear_session(&stream_session).await;
            crate::services::agent_local::session_store::remove_session_lock(&stream_session).await;
        }

        if let Err(message) = result {
            // Ne pas envoyer l'erreur "Annulé" — le frontend gère déjà le cancel
            // via stopSession(). Envoyer ce message tuerait un nouveau stream.
            if is_current && message != "Annulé" {
                let is_conn = message == "ollama_connection_lost";
                let _ = emitter.send(StreamEvent::Error { message, is_connection: is_conn });
            }
        }
    });

    Ok(generation)
}

#[tauri::command]
pub async fn cancel_agent_request(
    session_id: String,
    generation: Option<u64>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    let mut map = streams.0.lock().await;
    if let Some((token, gen)) = map.get(&session_id) {
        if generation.is_none() || generation == Some(*gen) {
            let token = token.clone();
            let gen = *gen;
            map.remove(&session_id);
            drop(map);
            token.cancel();
            eprintln!("[cancel] session={session_id} gen={gen}");
        }
    }
    Ok(())
}
