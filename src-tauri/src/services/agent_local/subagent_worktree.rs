use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub async fn remove(worktree_path: &str) -> Result<(), String> {
    let path = PathBuf::from(worktree_path);
    let root = data_dir().join("subagent-worktrees");
    if !path.starts_with(&root) {
        return Err("Chemin worktree hors du répertoire géré".to_string());
    }
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(&path)
        .kill_on_drop(true)
        .output()
        .await;
    if path.exists() {
        let _ = tokio::fs::remove_dir_all(&path).await;
    }
    Ok(())
}

pub async fn create_for_child(project_path: &Path, child_id: &str) -> Result<PathBuf, String> {
    if !project_path.is_dir() {
        return Err("Projet introuvable".to_string());
    }
    let root = data_dir().join("subagent-worktrees");
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|_| "Création du worktree impossible".to_string())?;
    if child_id.contains("..") || child_id.contains('/') || child_id.contains('\\') {
        return Err("ID de sous-agent invalide".to_string());
    }
    let target = root.join(child_id);
    if target.exists() {
        let _ = tokio::fs::remove_dir_all(&target).await;
    }

    let output = Command::new("git")
        .args(["-C"])
        .arg(project_path)
        .args(["worktree", "add", "--detach"])
        .arg(&target)
        .arg("HEAD")
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| "Git indisponible pour créer le worktree".to_string())?;

    if output.status.success() && target.is_dir() {
        Ok(target)
    } else {
        let _ = tokio::fs::remove_dir_all(&target).await;
        Err("Création du worktree isolé impossible".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::create_for_child;

    #[tokio::test]
    async fn test_reject_path_traversal_dotdot() {
        let tmp = std::env::temp_dir();
        let result = create_for_child(&tmp, "../../etc").await;
        assert!(result.is_err(), "child_id avec '..' doit être rejeté");
    }

    #[tokio::test]
    async fn test_reject_path_traversal_slash() {
        let tmp = std::env::temp_dir();
        let result = create_for_child(&tmp, "foo/bar").await;
        assert!(result.is_err(), "child_id avec '/' doit être rejeté");
    }

    #[tokio::test]
    async fn test_reject_path_traversal_backslash() {
        let tmp = std::env::temp_dir();
        let result = create_for_child(&tmp, "foo\\bar").await;
        assert!(result.is_err(), "child_id avec '\\\\' doit être rejeté");
    }
}
