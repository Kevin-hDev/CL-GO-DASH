use super::{
    clone_git_checks::{
        branch_linked_by_other_session, clone_linked_branch, ensure_clone_belongs_to_root,
        ensure_clone_belongs_to_root_string,
    },
    session_store, session_tabs,
};
use crate::services::git::{branch, branch_delete};
use rand::RngCore;
use serde::Serialize;
use std::path::{Path, PathBuf};

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
    ensure_clone_belongs_to_root(&clone, root_session_id).await?;
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

    let branch_name = create_unique_branch(repo_path).await?;
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
    ensure_clone_belongs_to_root_string(&clone, root_session_id).await?;
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
    let clone = session_store::get(&tab.session_id).await?;
    let Some(git_branch) = clone_linked_branch(&clone, root_session_id).await? else {
        return session_tabs::close_tab(root_session_id, tab_id).await;
    };

    // Fallback explicite du frontend, sinon checkpoint persisté dans session-tabs.json.
    let checkpoint = match fallback_branch.map(str::to_string) {
        Some(branch) => Some(branch),
        None => session_tabs::get_main_checkpoint_branch(root_session_id).await?,
    };

    if branch_linked_by_other_session(&git_branch, &tab.session_id).await? {
        unlink_branch(root_session_id, &tab.session_id).await?;
        return session_tabs::close_tab(root_session_id, tab_id).await;
    }

    // Switch vers le fallback si on était sur la branche clone, puis suppression.
    // Tout est synchro disque → spawn_blocking pour ne pas bloquer le runtime Tokio.
    let repo_path: PathBuf = repo_path.to_path_buf();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let context = branch::get_context(&repo_path);
        if context.branch == git_branch {
            let Some(fallback) = checkpoint.as_deref() else {
                return Err("Action impossible".to_string());
            };
            if fallback == git_branch {
                return Err("Action impossible".to_string());
            }
            if !branch_delete::branch_exists(&repo_path, fallback)? {
                return Err("Branche introuvable".to_string());
            }
            branch::checkout_branch(&repo_path, fallback)?;
        }
        if branch_delete::branch_exists(&repo_path, &git_branch)? {
            branch_delete::delete_branch(&repo_path, &git_branch)?;
        }
        Ok(())
    })
    .await
    .map_err(|e| {
        eprintln!("[clone-git] close cleanup join: {e}");
        "Erreur interne".to_string()
    })??;

    unlink_branch(root_session_id, &tab.session_id).await?;
    session_tabs::close_tab(root_session_id, tab_id).await
}

async fn create_unique_branch(repo_path: &Path) -> Result<String, branch::CreateBranchError> {
    let candidates: Vec<String> = (0..MAX_BRANCH_RETRIES)
        .map(|_| random_branch_name())
        .collect();
    create_unique_branch_from_candidates(repo_path.to_path_buf(), candidates).await
}

async fn create_unique_branch_from_candidates(
    repo_path: PathBuf,
    candidates: Vec<String>,
) -> Result<String, branch::CreateBranchError> {
    let mut last_error = branch::CreateBranchError::AlreadyExists;
    for branch_name in candidates.into_iter().take(MAX_BRANCH_RETRIES) {
        branch::validate_branch_name(&branch_name)?;
        let repo_for_create = repo_path.clone();
        let name_for_create = branch_name.clone();
        let result = tokio::task::spawn_blocking(move || {
            branch::create_branch(&repo_for_create, &name_for_create)
        })
        .await
        .map_err(|e| {
            eprintln!("[clone-git] create_branch join: {e}");
            branch::CreateBranchError::InternalError
        })?;
        match result {
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

#[cfg(test)]
#[path = "clone_git_tests.rs"]
mod tests;
