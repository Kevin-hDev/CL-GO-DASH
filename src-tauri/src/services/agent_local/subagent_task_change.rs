use std::path::Path;

pub async fn capture(
    project_path: &Path,
    child_id: &str,
    execution_id: &str,
    worktree: &Path,
) -> Result<Option<String>, String> {
    let Some(meta) = super::subagent_git_run::capture(
        project_path,
        child_id,
        execution_id,
        worktree,
    )
    .await?
    else {
        return Ok(None);
    };
    let metadata = serde_json::to_string(&meta)
        .map_err(|_| "Métadonnées de changement indisponibles".to_string())?;
    Ok(Some(format!(
        "\n\n<subagent_change_metadata>\n{metadata}\n</subagent_change_metadata>"
    )))
}

pub async fn delete_empty_branch(project_path: &Path, execution_id: &str) {
    let Ok(branch) = super::subagent_worktree::branch_for_execution(execution_id) else {
        return;
    };
    let _ = super::subagent_git_command::delete_branch(project_path, &branch).await;
}

pub async fn recover_and_remove_orphan(
    session: &super::types_session::AgentSession,
) -> Result<(), String> {
    let Some(worktree) = session.subagent_worktree.as_deref() else {
        return Ok(());
    };
    if session.subagent_type.as_deref() != Some("coder") {
        return super::subagent_worktree::remove_for_child(worktree, &session.id).await;
    }
    let identity = super::subagent_worktree_identity::ManagedWorktreeIdentity::parse(worktree)?;
    identity.require_child(&session.id)?;
    let project = super::project_store::list()
        .await
        .unwrap_or_default()
        .into_iter()
        .find(|project| Some(project.id.as_str()) == session.project_id.as_deref());
    let mut retain_branch = true;
    if let Some(project) = project.as_ref() {
        retain_branch = capture(
            Path::new(&project.path),
            &session.id,
            &identity.execution_id,
            &identity.path,
        )
        .await
        .ok()
        .flatten()
        .is_some();
    }
    super::subagent_worktree::remove_for_child(worktree, &session.id).await?;
    if !retain_branch {
        if let Some(project) = project {
            delete_empty_branch(Path::new(&project.path), &identity.execution_id).await;
        }
    }
    Ok(())
}
