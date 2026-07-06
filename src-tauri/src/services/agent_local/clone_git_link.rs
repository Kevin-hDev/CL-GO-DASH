use super::{clone_git_checks::ensure_clone_belongs_to_root_string, session_store, session_tabs};
use crate::services::git::{branch, branch_delete};
use std::path::Path;

pub async fn link_existing_branch(
    root_session_id: &str,
    clone_session_id: &str,
    repo_path: &Path,
    branch_name: &str,
) -> Result<session_tabs::SessionTabs, String> {
    branch::validate_branch_name(branch_name).map_err(|_| "Action impossible".to_string())?;
    if !branch_delete::branch_exists(repo_path, branch_name)? {
        return Err("Action impossible".into());
    }

    let mut clone = session_store::get(clone_session_id).await?;
    ensure_clone_belongs_to_root_string(&clone, root_session_id).await?;
    clone.git_branch = Some(branch_name.to_string());
    session_store::save(&clone).await?;
    session_tabs::set_clone_git_branch(
        root_session_id,
        clone_session_id,
        Some(branch_name.to_string()),
    )
    .await
}

#[cfg(test)]
#[path = "clone_git_link_tests.rs"]
mod tests;
