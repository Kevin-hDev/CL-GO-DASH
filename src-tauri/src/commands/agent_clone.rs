use crate::services::agent_local::clone_session::CloneSessionResult;
use crate::services::agent_local::session_tabs::SessionTabs;
use crate::services::agent_local::types_session::CloneMode;

#[tauri::command]
pub async fn clone_agent_session(
    session_id: String,
    message_id: String,
    mode: CloneMode,
    custom_focus: Option<String>,
    operation_id: Option<String>,
) -> Result<CloneSessionResult, String> {
    crate::services::agent_local::clone_session::clone_session(
        &session_id,
        &message_id,
        mode,
        custom_focus,
        operation_id,
    )
    .await
}

#[tauri::command]
pub async fn cancel_clone_summary(operation_id: String) -> Result<(), String> {
    crate::services::agent_local::clone_session::cancel_clone_summary(&operation_id).await
}

#[tauri::command]
pub async fn list_session_tabs(session_id: String) -> Result<SessionTabs, String> {
    crate::services::agent_local::session_tabs::list(&session_id).await
}

#[tauri::command]
pub async fn save_session_tabs(session_id: String, tabs: SessionTabs) -> Result<SessionTabs, String> {
    crate::services::agent_local::session_tabs::save_tabs(&session_id, tabs).await
}

#[tauri::command]
pub async fn close_session_tab(session_id: String, tab_id: String) -> Result<SessionTabs, String> {
    crate::services::agent_local::session_tabs::close_tab(&session_id, &tab_id).await
}

#[tauri::command]
pub async fn rename_session_tab(
    session_id: String,
    tab_id: String,
    label: String,
) -> Result<SessionTabs, String> {
    crate::services::agent_local::session_tabs::rename_tab(&session_id, &tab_id, &label).await
}
