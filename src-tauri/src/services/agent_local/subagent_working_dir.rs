use crate::services::agent_local::session_store;
use std::path::{Path, PathBuf};

pub struct PreparedWorkingDir {
    path: PathBuf,
    worktree_path: Option<String>,
}

impl PreparedWorkingDir {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn worktree_path(&self) -> Option<&str> {
        self.worktree_path.as_deref()
    }
}

pub async fn resolve(
    project_id: Option<&str>,
    child_session_id: &str,
    is_explorer: bool,
    run_id: &str,
    execution_id: &str,
) -> Result<PreparedWorkingDir, String> {
    if !is_explorer && project_id.is_none() {
        return Err("Un sous-agent code doit être lancé depuis un projet.".to_string());
    }
    let base = super::subagent_prompts::resolve_project_dir(project_id).await;
    if !is_explorer && project_id.is_some() && base != dirs::home_dir().unwrap_or_default() {
        return create_coder_worktree(&base, child_session_id, run_id, execution_id).await;
    }
    Ok(PreparedWorkingDir {
        path: base,
        worktree_path: None,
    })
}

async fn create_coder_worktree(
    base: &Path,
    child_session_id: &str,
    run_id: &str,
    execution_id: &str,
) -> Result<PreparedWorkingDir, String> {
    let lock = session_store::lock_session(child_session_id).await;
    let _guard = lock.lock().await;
    if !super::subagent_registry::owns_execution(child_session_id, run_id, execution_id).await {
        return Err("Préparation du worktree isolé impossible".to_string());
    }
    let _git_guard = super::subagent_git_lock::acquire(base).await?;
    let worktree = super::subagent_worktree::create_for_execution(
        base,
        child_session_id,
        execution_id,
    )
    .await?;
    let path = worktree.to_string_lossy().to_string();
    if super::subagent_git_run::seed_pending_locked(base, child_session_id, execution_id, &worktree)
        .await
        .is_err()
    {
        cleanup_failed(base, &path, child_session_id, execution_id).await;
        return Err("Préparation du worktree isolé impossible".to_string());
    }
    let mut session = match session_store::get(child_session_id).await {
        Ok(session) => session,
        Err(_) => {
            cleanup_failed(base, &path, child_session_id, execution_id).await;
            return Err("Préparation du worktree isolé impossible".to_string());
        }
    };
    let owns_execution = session.subagent_run_id.as_deref() == Some(run_id)
        && super::subagent_registry::owns_execution(child_session_id, run_id, execution_id).await;
    if !owns_execution {
        cleanup_failed(base, &path, child_session_id, execution_id).await;
        return Err("Préparation du worktree isolé impossible".to_string());
    }
    session.subagent_worktree = Some(path.clone());
    if session_store::save(&session).await.is_err() {
        cleanup_failed(base, &path, child_session_id, execution_id).await;
        return Err("Préparation du worktree isolé impossible".to_string());
    }
    Ok(PreparedWorkingDir {
        path: worktree,
        worktree_path: Some(path),
    })
}

async fn cleanup_failed(base: &Path, path: &str, child_id: &str, execution_id: &str) {
    let _ = super::subagent_worktree::remove_owned(path, child_id, execution_id).await;
    if let Ok(branch) = super::subagent_worktree::branch_for_execution(execution_id) {
        let _ = super::subagent_git_command::delete_branch(base, &branch).await;
    }
}

pub async fn cleanup_owned(
    child_session_id: &str,
    expected_execution_id: &str,
    expected_worktree_path: Option<&str>,
) {
    let Some(expected) = expected_worktree_path else {
        return;
    };
    let lock = session_store::lock_session(child_session_id).await;
    let _guard = lock.lock().await;
    let Ok(mut session) = session_store::get(child_session_id).await else {
        return;
    };
    let clear_current = session.subagent_worktree.as_deref() == Some(expected);
    if super::subagent_worktree::remove_owned(
        expected,
        child_session_id,
        expected_execution_id,
    )
    .await
    .is_err()
    {
        eprintln!("[subagent] cleanup worktree");
        return;
    }
    if clear_current {
        session.subagent_worktree = None;
        let _ = session_store::save(&session).await;
    }
}

#[cfg(test)]
pub async fn create_coder_worktree_for_test(
    base: &Path,
    child_session_id: &str,
    run_id: &str,
    execution_id: &str,
) -> Result<PreparedWorkingDir, String> {
    create_coder_worktree(base, child_session_id, run_id, execution_id).await
}
