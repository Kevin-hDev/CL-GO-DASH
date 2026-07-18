use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use super::{branch_commit, status, worktree_list};

const MAX_PATH_CHARS: usize = 4_096;
const REMOVE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize)]
pub struct WorktreeDeletePreview {
    pub path: String,
    pub branch: String,
    pub dirty_files: Vec<status::DirtyFile>,
}

pub async fn preview(
    repo_path: &Path,
    target_path: &Path,
) -> Result<WorktreeDeletePreview, String> {
    let (target, info) = validate_target(repo_path, target_path).await?;
    let dirty_target = target.clone();
    let dirty_files = tokio::task::spawn_blocking(move || status::list_dirty_files(&dirty_target))
        .await
        .map_err(|_| "Erreur interne".to_string())??;
    Ok(WorktreeDeletePreview {
        path: target.to_string_lossy().to_string(),
        branch: info.branch,
        dirty_files,
    })
}

pub async fn remove_clean(repo_path: &Path, target_path: &Path) -> Result<(), String> {
    let (target, _) = validate_target(repo_path, target_path).await?;
    remove(repo_path, &target, false).await
}

pub async fn discard_and_remove(repo_path: &Path, target_path: &Path) -> Result<(), String> {
    let (target, _) = validate_target(repo_path, target_path).await?;
    remove(repo_path, &target, true).await
}

pub async fn preserve_and_remove(
    repo_path: &Path,
    target_path: &Path,
    description: Option<String>,
) -> Result<(), String> {
    let (target, _) = validate_target(repo_path, target_path).await?;
    let commit_target = target.clone();
    tokio::task::spawn_blocking(move || branch_commit::commit_all(&commit_target, description))
        .await
        .map_err(|_| "Erreur interne".to_string())??;
    remove(repo_path, &target, false).await
}

async fn validate_target(
    repo_path: &Path,
    target_path: &Path,
) -> Result<(PathBuf, worktree_list::WorktreeInfo), String> {
    if target_path.as_os_str().is_empty()
        || target_path.to_string_lossy().chars().count() > MAX_PATH_CHARS
    {
        return Err("Worktree invalide".to_string());
    }
    let target = std::fs::canonicalize(target_path).map_err(|_| "Worktree invalide".to_string())?;
    let worktrees = worktree_list::list_worktrees(repo_path).await?;
    let info = worktrees
        .into_iter()
        .find(|worktree| {
            !worktree.is_current
                && std::fs::canonicalize(&worktree.path).ok().as_ref() == Some(&target)
        })
        .ok_or_else(|| "Worktree invalide".to_string())?;
    Ok((target, info))
}

async fn remove(repo_path: &Path, target_path: &Path, force: bool) -> Result<(), String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo_path)
        .args(["worktree", "remove"]);
    if force {
        command.args(["--force", "--force"]);
    }
    command
        .arg(target_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let result = timeout(REMOVE_TIMEOUT, command.status())
        .await
        .map_err(|_| "Suppression du worktree impossible".to_string())?
        .map_err(|_| "Suppression du worktree impossible".to_string())?;
    if !result.success() {
        return Err("Suppression du worktree impossible".to_string());
    }
    Ok(())
}
