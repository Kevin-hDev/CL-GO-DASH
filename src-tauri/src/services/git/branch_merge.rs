use std::path::Path;
use std::process::{Command, Stdio};

use super::branch;

pub(super) fn merge_branch(repo_path: &Path, branch_name: &str) -> Result<(), String> {
    branch::validate_branch_name(branch_name).map_err(|e| e.to_string())?;
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["merge", "--no-edit", "--no-verify", branch_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| "Fusion impossible".to_string())?;
    if status.success() {
        return Ok(());
    }
    let _ = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["merge", "--abort"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    Err("Fusion impossible".to_string())
}
