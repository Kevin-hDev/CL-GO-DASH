use crate::services::agent_local::session_store;
use std::path::{Path, PathBuf};

pub async fn resolve(
    project_id: Option<&str>,
    child_session_id: &str,
    is_explorer: bool,
) -> Result<PathBuf, String> {
    let base = super::subagent_prompts::resolve_project_dir(project_id).await;
    if !is_explorer && project_id.is_some() && base != dirs::home_dir().unwrap_or_default() {
        return create_coder_worktree(&base, child_session_id).await;
    }
    Ok(base)
}

async fn create_coder_worktree(base: &Path, child_session_id: &str) -> Result<PathBuf, String> {
    let worktree = super::subagent_worktree::create_for_child(base, child_session_id).await?;
    let save_result = async {
        let mut session = session_store::get(child_session_id).await?;
        session.subagent_worktree = Some(worktree.to_string_lossy().to_string());
        session_store::save(&session).await
    }
    .await;

    if save_result.is_err() {
        let _ = super::subagent_worktree::remove(&worktree.to_string_lossy()).await;
        return Err("Préparation du worktree isolé impossible".to_string());
    }

    Ok(worktree)
}

pub async fn cleanup(child_session_id: &str) {
    if let Ok(session) = session_store::get(child_session_id).await {
        if let Some(wt_path) = &session.subagent_worktree {
            if let Err(e) = super::subagent_worktree::remove(wt_path).await {
                eprintln!("[subagent] cleanup worktree: {e}");
            }
        }
    }
}

#[cfg(test)]
pub async fn create_coder_worktree_for_test(
    base: &Path,
    child_session_id: &str,
) -> Result<PathBuf, String> {
    create_coder_worktree(base, child_session_id).await
}
