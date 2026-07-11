use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

pub async fn remove(worktree_path: &str) -> Result<(), String> {
    if super::subagent_worktree::has_forbidden_component(worktree_path) {
        return Err("Chemin worktree invalide".to_string());
    }
    let path = PathBuf::from(worktree_path);
    let root = data_dir().join("subagent-worktrees");
    let canonical_root = match tokio::fs::canonicalize(&root).await {
        Ok(root) => root,
        Err(_) => return Ok(()),
    };
    let child_dir = managed_child_dir(&path, &canonical_root).await;
    let removal = remove_managed_path(&path, &canonical_root).await;
    let parent_cleanup = cleanup_empty_parent(child_dir.as_deref()).await;

    removal.and(parent_cleanup)
}

async fn remove_managed_path(path: &Path, canonical_root: &Path) -> Result<(), String> {
    let canonical_path = match tokio::fs::canonicalize(path).await {
        Ok(path) => path,
        Err(_) => match tokio::fs::symlink_metadata(path).await {
            Ok(_) => return Err("Chemin worktree invalide".to_string()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err("Chemin worktree invalide".to_string()),
        },
    };
    if canonical_path == canonical_root || !canonical_path.starts_with(canonical_root) {
        return Err("Chemin worktree hors du répertoire géré".to_string());
    }

    if git_remove(&canonical_path, false).await || !canonical_path.exists() {
        return verify_removed(&canonical_path);
    }
    if git_remove(&canonical_path, true).await || !canonical_path.exists() {
        return verify_removed(&canonical_path);
    }
    remove_partial_directory(&canonical_path).await?;
    verify_removed(&canonical_path)
}

async fn git_remove(path: &Path, retry_locked: bool) -> bool {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(path)
        .args(["worktree", "remove", "--force"]);
    if retry_locked {
        command.arg("--force");
    }
    command
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false)
}

async fn remove_partial_directory(path: &Path) -> Result<(), String> {
    if tokio::fs::symlink_metadata(path.join(".git")).await.is_ok() {
        return Err("Suppression du worktree impossible".to_string());
    }
    tokio::fs::remove_dir_all(path)
        .await
        .map_err(|_| "Suppression du worktree impossible".to_string())
}

async fn managed_child_dir(path: &Path, canonical_root: &Path) -> Option<PathBuf> {
    let parent = tokio::fs::canonicalize(path.parent()?).await.ok()?;
    if parent.parent()? != canonical_root {
        return None;
    }
    let child_id = parent.file_name()?.to_str()?;
    uuid::Uuid::parse_str(child_id).ok()?;
    Some(parent)
}

async fn cleanup_empty_parent(child_dir: Option<&Path>) -> Result<(), String> {
    let Some(child_dir) = child_dir else {
        return Ok(());
    };
    match tokio::fs::remove_dir(child_dir).await {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(_) => Err("Nettoyage du répertoire worktree impossible".to_string()),
    }
}

fn verify_removed(path: &Path) -> Result<(), String> {
    if path.exists() {
        Err("Suppression du worktree impossible".to_string())
    } else {
        Ok(())
    }
}
