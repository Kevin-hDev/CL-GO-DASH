use crate::services::agent_local::session_store;

pub async fn mark_status(session_id: &str, status: &str) -> Result<(), String> {
    if !matches!(status, "running" | "completed" | "failed" | "cancelled") {
        return Err("Statut sous-agent invalide".to_string());
    }
    let mut session = session_store::get(session_id).await?;
    session.subagent_status = Some(status.to_string());
    session_store::save(&session).await
}
