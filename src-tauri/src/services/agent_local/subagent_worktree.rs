use crate::services::paths::data_dir;
use std::path::{Component, Path, PathBuf};
use tokio::process::Command;

pub async fn remove(worktree_path: &str) -> Result<(), String> {
    if has_forbidden_component(worktree_path) {
        return Err("Chemin worktree invalide".to_string());
    }
    let path = PathBuf::from(worktree_path);
    let root = data_dir().join("subagent-worktrees");
    let canonical_root = match tokio::fs::canonicalize(&root).await {
        Ok(root) => root,
        Err(_) => return Ok(()),
    };
    let canonical_path = match tokio::fs::canonicalize(&path).await {
        Ok(path) => path,
        Err(_) => match tokio::fs::symlink_metadata(&path).await {
            Ok(_) => return Err("Chemin worktree invalide".to_string()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err("Chemin worktree invalide".to_string()),
        },
    };
    if canonical_path == canonical_root || !canonical_path.starts_with(&canonical_root) {
        return Err("Chemin worktree hors du répertoire géré".to_string());
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

fn has_forbidden_component(path: &str) -> bool {
    path.contains('\0')
        || Path::new(path)
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

pub fn path_for_execution(child_id: &str, execution_id: &str) -> Result<PathBuf, String> {
    if uuid::Uuid::parse_str(child_id).is_err() || uuid::Uuid::parse_str(execution_id).is_err() {
        return Err("ID de sous-agent invalide".to_string());
    }
    Ok(data_dir()
        .join("subagent-worktrees")
        .join(child_id)
        .join(execution_id))
}

pub async fn create_for_execution(
    project_path: &Path,
    child_id: &str,
    execution_id: &str,
) -> Result<PathBuf, String> {
    if !project_path.is_dir() {
        return Err("Projet introuvable".to_string());
    }
    let target = path_for_execution(child_id, execution_id)?;
    let parent = target
        .parent()
        .ok_or_else(|| "Chemin worktree invalide".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|_| "Création du worktree impossible".to_string())?;
    match tokio::fs::symlink_metadata(&target).await {
        Ok(_) => return Err("Collision de worktree isolé".to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err("Création du worktree impossible".to_string()),
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
        let _ = remove(&target.to_string_lossy()).await;
        Err("Création du worktree isolé impossible".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::create_for_execution;

    fn execution_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    #[tokio::test]
    async fn test_reject_path_traversal_dotdot() {
        let tmp = std::env::temp_dir();
        let result = create_for_execution(&tmp, "../../etc", &execution_id()).await;
        assert!(result.is_err(), "child_id avec '..' doit être rejeté");
    }

    #[tokio::test]
    async fn test_reject_path_traversal_slash() {
        let tmp = std::env::temp_dir();
        let result = create_for_execution(&tmp, "foo/bar", &execution_id()).await;
        assert!(result.is_err(), "child_id avec '/' doit être rejeté");
    }

    #[tokio::test]
    async fn test_reject_path_traversal_backslash() {
        let tmp = std::env::temp_dir();
        let result = create_for_execution(&tmp, "foo\\bar", &execution_id()).await;
        assert!(result.is_err(), "child_id avec '\\\\' doit être rejeté");
    }

    #[tokio::test]
    async fn test_reject_empty_child_id() {
        let tmp = std::env::temp_dir();
        let result = create_for_execution(&tmp, "", &execution_id()).await;
        assert!(result.is_err(), "child_id vide doit être rejeté");
    }

    #[test]
    fn test_raw_path_rejects_parent_dir_component() {
        assert!(super::has_forbidden_component(
            "/tmp/subagent-worktrees/child/../../outside"
        ));
    }
}
