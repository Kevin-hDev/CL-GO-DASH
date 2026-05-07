use crate::services::agent_local::session_store;

pub async fn mark_status(session_id: &str, status: &str) -> Result<(), String> {
    if !matches!(status, "running" | "completed" | "failed" | "cancelled") {
        return Err("Statut sous-agent invalide".to_string());
    }
    let mut session = session_store::get(session_id).await?;
    session.subagent_status = Some(status.to_string());
    session_store::save(&session).await
}

#[cfg(test)]
mod tests {
    use super::mark_status;

    #[tokio::test]
    async fn test_reject_invalid_status() {
        // La validation du statut est avant l'accès filesystem.
        let result = mark_status("any-id", "invalid_status").await;
        assert!(result.is_err(), "Un statut invalide doit être rejeté");
        assert_eq!(result.unwrap_err(), "Statut sous-agent invalide");
    }
}
