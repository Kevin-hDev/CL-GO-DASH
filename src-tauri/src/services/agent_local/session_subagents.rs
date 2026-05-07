use crate::services::agent_local::session_store;
use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};

fn is_managed_worktree(path: &Path) -> bool {
    path.starts_with(data_dir().join("subagent-worktrees"))
}

async fn remove_managed_worktree(path: Option<String>) {
    let Some(raw) = path else {
        return;
    };
    let path = PathBuf::from(raw);
    if is_managed_worktree(&path) {
        let _ = tokio::fs::remove_dir_all(path).await;
    }
}

pub async fn delete_old_for_run(parent_id: &str, run_id: &str) -> Result<(), String> {
    let sessions = session_store::list().await?;
    for meta in sessions {
        if meta.parent_session_id.as_deref() != Some(parent_id) {
            continue;
        }
        if meta.subagent_run_id.as_deref() == Some(run_id) {
            continue;
        }
        if let Ok(session) = session_store::get(&meta.id).await {
            remove_managed_worktree(session.subagent_worktree).await;
        }
        let _ = session_store::delete(&meta.id).await;
    }
    Ok(())
}

pub async fn mark_status(session_id: &str, status: &str) -> Result<(), String> {
    if !matches!(status, "running" | "completed" | "failed" | "cancelled") {
        return Err("Statut sous-agent invalide".to_string());
    }
    let mut session = session_store::get(session_id).await?;
    session.subagent_status = Some(status.to_string());
    session_store::save(&session).await
}
