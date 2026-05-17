use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub async fn remove(worktree_path: &str) -> Result<(), String> {
    let path = PathBuf::from(worktree_path);
    let root = data_dir().join("subagent-worktrees");
    let canonical_root = canonical_or_original(&root).await;
    let canonical_path = canonical_or_original(&path).await;
    if !canonical_path.starts_with(&canonical_root) {
        return Err("Chemin worktree hors du répertoire géré".to_string());
    }
    if !canonical_path.exists() {
        return Ok(());
    }

    let output = Command::new("git")
        .args(["-C"])
        .arg(&canonical_path)
        .args(["worktree", "remove", "--force"])
        .arg(&canonical_path)
        .kill_on_drop(true)
        .output()
        .await;

    if output.map(|o| !o.status.success()).unwrap_or(true) && canonical_path.exists() {
        tokio::fs::remove_dir_all(&canonical_path)
            .await
            .map_err(|_| "Suppression du worktree impossible".to_string())?;
    }

    if canonical_path.exists() {
        return Err("Suppression du worktree impossible".to_string());
    }

    Ok(())
}

async fn canonical_or_original(path: &Path) -> PathBuf {
    tokio::fs::canonicalize(path)
        .await
        .unwrap_or_else(|_| path.to_path_buf())
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
