use super::{session_store, session_tabs};
use crate::services::git::{branch, branch_delete};
use rand::RngCore;
use serde::Serialize;
use std::path::Path;

const MAX_BRANCH_RETRIES: usize = 3;

#[derive(Debug, Clone, Serialize)]
pub struct CloneGitBranchResult {
    pub branch_name: String,
    pub tabs: session_tabs::SessionTabs,
}

pub async fn create_linked_branch(
    root_session_id: &str,
    clone_session_id: &str,
    repo_path: &Path,
) -> Result<CloneGitBranchResult, branch::CreateBranchError> {
    let mut clone = session_store::get(clone_session_id)
        .await
        .map_err(|_| branch::CreateBranchError::InternalError)?;
    ensure_clone_belongs_to_root(&clone, root_session_id)?;
    if let Some(branch_name) = clone.git_branch.clone() {
        let tabs = session_tabs::set_clone_git_branch(
            root_session_id,
            clone_session_id,
            Some(branch_name.clone()),
        )
        .await
        .map_err(|_| branch::CreateBranchError::InternalError)?;
        return Ok(CloneGitBranchResult { branch_name, tabs });
    }

    let branch_name = create_unique_branch(repo_path)?;
    clone.git_branch = Some(branch_name.clone());
    session_store::save(&clone)
        .await
        .map_err(|_| branch::CreateBranchError::InternalError)?;
    let tabs = session_tabs::set_clone_git_branch(
        root_session_id,
        clone_session_id,
        Some(branch_name.clone()),
    )
    .await
    .map_err(|_| branch::CreateBranchError::InternalError)?;
    Ok(CloneGitBranchResult { branch_name, tabs })
}

pub async fn unlink_branch(
    root_session_id: &str,
    clone_session_id: &str,
) -> Result<session_tabs::SessionTabs, String> {
    let mut clone = session_store::get(clone_session_id).await?;
    ensure_clone_belongs_to_root_string(&clone, root_session_id)?;
    clone.git_branch = None;
    session_store::save(&clone).await?;
    session_tabs::set_clone_git_branch(root_session_id, clone_session_id, None).await
}

pub async fn close_tab_with_branch_cleanup(
    root_session_id: &str,
    tab_id: &str,
    repo_path: &Path,
    fallback_branch: Option<&str>,
) -> Result<session_tabs::SessionTabs, String> {
    let tab = session_tabs::get_tab(root_session_id, tab_id).await?;
    let Some(git_branch) = tab.git_branch.as_deref() else {
        return session_tabs::close_tab(root_session_id, tab_id).await;
    };
    checkout_fallback_if_needed(repo_path, git_branch, fallback_branch)?;
    if branch_delete::branch_exists(repo_path, git_branch)? {
        branch_delete::delete_branch(repo_path, git_branch)?;
    }
    unlink_branch(root_session_id, &tab.session_id).await?;
    session_tabs::close_tab(root_session_id, tab_id).await
}

fn create_unique_branch(repo_path: &Path) -> Result<String, branch::CreateBranchError> {
    create_unique_branch_from_candidates(repo_path, (0..MAX_BRANCH_RETRIES).map(|_| random_branch_name()))
}

fn create_unique_branch_from_candidates<I>(
    repo_path: &Path,
    candidates: I,
) -> Result<String, branch::CreateBranchError>
where
    I: IntoIterator<Item = String>,
{
    let mut last_error = branch::CreateBranchError::AlreadyExists;
    for branch_name in candidates.into_iter().take(MAX_BRANCH_RETRIES) {
        branch::validate_branch_name(&branch_name)?;
        match branch::create_branch(repo_path, &branch_name) {
            Ok(()) => return Ok(branch_name),
            Err(branch::CreateBranchError::AlreadyExists) => {
                last_error = branch::CreateBranchError::AlreadyExists;
            }
            Err(err) => return Err(err),
        }
    }
    Err(last_error)
}

fn random_branch_name() -> String {
    let mut bytes = [0_u8; 4];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    format!("clone-{}", hex_lower(&bytes))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn checkout_fallback_if_needed(
    repo_path: &Path,
    git_branch: &str,
    fallback_branch: Option<&str>,
) -> Result<(), String> {
    let context = branch::get_context(repo_path);
    if context.branch != git_branch {
        return Ok(());
    }
    let fallback = fallback_branch.ok_or_else(|| "Action impossible".to_string())?;
    if fallback == git_branch {
        return Err("Action impossible".into());
    }
    branch::checkout_branch(repo_path, fallback)
}

fn ensure_clone_belongs_to_root(
    clone: &super::types_session::AgentSession,
    root_session_id: &str,
) -> Result<(), branch::CreateBranchError> {
    if clone.clone_parent_session_id.as_deref() == Some(root_session_id) {
        Ok(())
    } else {
        Err(branch::CreateBranchError::InternalError)
    }
}

fn ensure_clone_belongs_to_root_string(
    clone: &super::types_session::AgentSession,
    root_session_id: &str,
) -> Result<(), String> {
    if clone.clone_parent_session_id.as_deref() == Some(root_session_id) {
        Ok(())
    } else {
        Err("Action impossible".into())
    }
}

#[cfg(test)]
#[path = "clone_git_tests.rs"]
mod tests;
