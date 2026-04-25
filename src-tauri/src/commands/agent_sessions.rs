use crate::services::agent_local::types_session::{
    AgentMessage, AgentSession, AgentSessionMeta, TabState,
};
use crate::services::agent_local::{session_store, tab_store};

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
    messages: Vec<AgentMessage>,
    tokens: u32,
) -> Result<(), String> {
    session_store::add_messages(&id, messages, tokens).await
}

#[tauri::command]
pub async fn create_agent_session(
    name: String,
    model: String,
    provider: Option<String>,
    project_id: Option<String>,
) -> Result<AgentSession, String> {
    let provider = provider.unwrap_or_else(|| "ollama".to_string());
    session_store::create_full(&name, &model, &provider, false, project_id).await
}

#[tauri::command]
pub async fn rename_agent_session(id: String, name: String) -> Result<(), String> {
    session_store::rename(&id, &name).await
}

#[tauri::command]
pub async fn update_session_model(id: String, model: String, provider: String) -> Result<(), String> {
    session_store::update_model(&id, &model, &provider).await
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
pub async fn truncate_session_at(session_id: String, message_id: String) -> Result<(), String> {
    session_store::truncate_at(&session_id, &message_id).await
}

#[tauri::command]
pub async fn truncate_and_replace_at(
    session_id: String,
    message_id: String,
    replacement: Option<AgentMessage>,
) -> Result<(), String> {
    session_store::truncate_and_replace(&session_id, &message_id, replacement).await
}

#[tauri::command]
pub async fn get_tab_state() -> Result<TabState, String> {
    tab_store::get_state().await
}

#[tauri::command]
pub async fn save_tab_state(state: TabState) -> Result<(), String> {
    tab_store::save_state(&state).await
}
