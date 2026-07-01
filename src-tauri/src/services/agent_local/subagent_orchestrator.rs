use crate::services::agent_local::session_store;
use crate::services::agent_local::types_session::AgentMessage;

pub(super) async fn inject_summary_in_parent(
    parent_session_id: &str,
    child_session_id: &str,
    child_name: &str,
    summary: &str,
    success: bool,
) {
    let status_label = if success { "terminé" } else { "échoué" };
    let content = format!(
        "[Rapport du sous-agent \"{child_name}\" — {status_label} — sid:{child_session_id}]\n\n{summary}"
    );
    let msg = AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
    };
    let _ = session_store::add_messages(parent_session_id, vec![msg], 0).await;
}

pub(super) async fn get_child_name(child_id: &str) -> String {
    session_store::get(child_id)
        .await
        .map(|s| s.name.clone())
        .unwrap_or_else(|_| "agent".to_string())
}
