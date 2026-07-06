use super::{clone_git_checks::clone_linked_branch, session_store, session_tabs};
use crate::services::git::{branch, branch_delete};
use std::path::Path;

pub async fn close_tab_with_branch_cleanup(
    root_session_id: &str,
    tab_id: &str,
    repo_path: &Path,
    fallback_branch: Option<&str>,
) -> Result<session_tabs::SessionTabs, String> {
    let tab = session_tabs::get_tab(root_session_id, tab_id).await?;
    let clone = session_store::get(&tab.session_id).await?;
    let Some(git_branch) = clone_linked_branch(&clone, root_session_id).await? else {
        return session_tabs::close_tab(root_session_id, tab_id).await;
    };
    let linked_session_ids = linked_sessions_for_branch(&git_branch, &tab.session_id).await?;

    let checkpoint = fallback_branch
        .map(str::to_string)
        .or(session_tabs::get_main_checkpoint_branch(root_session_id).await?);
    let repo_path = repo_path.to_path_buf();
    let deleted_branch = git_branch.clone();
    let replacement_branch = tokio::task::spawn_blocking(move || {
        cleanup_linked_branch(&repo_path, &deleted_branch, checkpoint.as_deref())
    })
    .await
    .map_err(|e| {
        eprintln!("[clone-git] close cleanup join: {e}");
        "Erreur interne".to_string()
    })??;

    unlink_branch_from_sessions(&linked_session_ids).await?;
    session_tabs::replace_main_checkpoint_branch(&git_branch, &replacement_branch).await?;
    session_tabs::close_tab(root_session_id, tab_id).await
}

async fn linked_sessions_for_branch(
    branch_name: &str,
    current_session_id: &str,
) -> Result<Vec<String>, String> {
    let mut ids: Vec<String> = super::session_index::read_index()
        .await?
        .into_iter()
        .filter(|meta| meta.git_branch.as_deref() == Some(branch_name))
        .map(|meta| meta.id)
        .collect();
    if !ids.iter().any(|id| id == current_session_id) {
        ids.push(current_session_id.to_string());
    }
    Ok(ids)
}

async fn unlink_branch_from_sessions(session_ids: &[String]) -> Result<(), String> {
    for session_id in session_ids {
        let mut session = session_store::get(session_id).await?;
        session.git_branch = None;
        session_store::save(&session).await?;
    }
    session_tabs::clear_git_branch_for_sessions(session_ids).await
}

fn cleanup_linked_branch(
    repo_path: &Path,
    git_branch: &str,
    checkpoint: Option<&str>,
) -> Result<String, String> {
    if is_protected_base_branch(git_branch) {
        return Err("Action impossible".to_string());
    }
    let context = branch::get_context(repo_path);
    let replacement = if context.branch == git_branch {
        let fallback = choose_cleanup_fallback(repo_path, git_branch, checkpoint)?;
        branch::checkout_branch(repo_path, &fallback)?;
        fallback
    } else {
        context.branch
    };
    if branch_delete::branch_exists(repo_path, git_branch)? {
        branch_delete::delete_branch(repo_path, git_branch)?;
    }
    Ok(replacement)
}

fn choose_cleanup_fallback(
    repo_path: &Path,
    git_branch: &str,
    checkpoint: Option<&str>,
) -> Result<String, String> {
    for name in ["main", "master", "develop", "dev"] {
        if name != git_branch && branch_delete::branch_exists(repo_path, name)? {
            return Ok(name.to_string());
        }
    }
    if let Some(checkpoint) = checkpoint {
        if checkpoint != git_branch && branch_delete::branch_exists(repo_path, checkpoint)? {
            return Ok(checkpoint.to_string());
        }
    }
    let branch = branch::list_branches(repo_path)?
        .into_iter()
        .find(|branch| branch.name != git_branch)
        .map(|branch| branch.name)
        .ok_or_else(|| "Action impossible".to_string())?;
    Ok(branch)
}

fn is_protected_base_branch(branch_name: &str) -> bool {
    matches!(branch_name, "main" | "master" | "develop" | "dev")
}

#[cfg(test)]
#[path = "clone_git_cleanup_tests.rs"]
mod tests;
