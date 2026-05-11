use serde::Serialize;
use std::path::Path;
use tokio::process::Command;

const MAX_WORKTREES: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub is_current: bool,
}

pub async fn list_worktrees(repo_path: &Path) -> Result<Vec<WorktreeInfo>, String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo_path)
        .args(["worktree", "list", "--porcelain"])
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| "Git indisponible".to_string())?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let repo_str = repo_path.to_string_lossy();

    let mut worktrees = Vec::new();
    let mut current_path = String::new();
    let mut current_branch = String::new();
    let mut is_bare = false;

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = path.to_string();
            current_branch.clear();
            is_bare = false;
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = branch.to_string();
        } else if line == "bare" {
            is_bare = true;
        } else if line.is_empty() && !current_path.is_empty() {
            if !is_bare && current_path != *repo_str && worktrees.len() < MAX_WORKTREES {
                worktrees.push(WorktreeInfo {
                    path: current_path.clone(),
                    branch: current_branch.clone(),
                    is_current: false,
                });
            }
            current_path.clear();
        }
    }

    if !current_path.is_empty()
        && !is_bare
        && current_path != *repo_str
        && worktrees.len() < MAX_WORKTREES
    {
        worktrees.push(WorktreeInfo {
            path: current_path,
            branch: current_branch,
            is_current: false,
        });
    }

    Ok(worktrees)
}
