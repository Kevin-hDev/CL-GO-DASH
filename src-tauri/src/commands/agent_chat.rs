use crate::services::agent_local::agent_loop;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::{session_store, tab_store};
use crate::services::agent_local::types_ollama::{
    ChatMessage, StreamEvent,
};
use crate::services::agent_local::types_session::{
    AgentSession, AgentSessionMeta, TabState,
};
use crate::ActiveStreams;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn chat_stream(
    session_id: String,
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<serde_json::Value>,
    think: bool,
    on_event: Channel<StreamEvent>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    let cancel = CancellationToken::new();
    streams.0.lock().await.insert(session_id.clone(), cancel.clone());

    let final_tools = if tools.is_empty() {
        tool_dispatcher::get_tool_definitions()
    } else {
        tools
    };

    let working_dir = std::env::current_dir().unwrap_or_else(|_| dirs::home_dir().unwrap());
    let mut msgs = messages;

    let result = agent_loop::run_agent_loop(
        &on_event,
        &mut msgs,
        &model,
        final_tools,
        think,
        working_dir,
        cancel,
    )
    .await;

    streams.0.lock().await.remove(&session_id);
    result.map(|_| ())
}

#[tauri::command]
pub async fn cancel_agent_request(
    session_id: String,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    if let Some(token) = streams.0.lock().await.get(&session_id) {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn list_agent_sessions() -> Result<Vec<AgentSessionMeta>, String> {
    session_store::list().await
}

#[tauri::command]
pub async fn get_agent_session(id: String) -> Result<AgentSession, String> {
    session_store::get(&id).await
}

#[tauri::command]
pub async fn save_agent_session(session: AgentSession) -> Result<(), String> {
    session_store::save(&session).await
}

#[tauri::command]
pub async fn add_messages_to_session(
    id: String,
    messages: Vec<crate::services::agent_local::types_session::AgentMessage>,
    tokens: u32,
) -> Result<(), String> {
    session_store::add_messages(&id, messages, tokens).await
}

#[tauri::command]
pub async fn create_agent_session(
    name: String,
    model: String,
) -> Result<AgentSession, String> {
    session_store::create(&name, &model).await
}

#[tauri::command]
pub async fn rename_agent_session(id: String, name: String) -> Result<(), String> {
    session_store::rename(&id, &name).await
}

#[tauri::command]
pub async fn delete_agent_session(id: String) -> Result<(), String> {
    session_store::delete(&id).await
}

#[tauri::command]
pub async fn export_agent_session_markdown(id: String) -> Result<String, String> {
    session_store::export_markdown(&id).await
}

#[tauri::command]
pub async fn truncate_session_at(
    session_id: String,
    message_id: String,
) -> Result<(), String> {
    session_store::truncate_at(&session_id, &message_id).await
}

#[tauri::command]
pub async fn get_tab_state() -> Result<TabState, String> {
    tab_store::get_state().await
}

#[tauri::command]
pub async fn save_tab_state(state: TabState) -> Result<(), String> {
    tab_store::save_state(&state).await
}
