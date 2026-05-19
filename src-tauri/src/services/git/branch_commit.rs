use git2::{BranchType, IndexAddOption, Repository, Signature};
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{branch, repo};

const MAX_COMMIT_DESCRIPTION_CHARS: usize = 2_000;

pub fn commit_all_and_checkout(
    repo_path: &Path,
    target_branch: &str,
    description: Option<String>,
) -> Result<(), String> {
    branch::validate_branch_name(target_branch)?;
    let git_repo = repo::open(repo_path)?;
    let workdir = repo::workdir(&git_repo)?;
    ensure_local_branch_exists(&git_repo, target_branch)?;

    let dirty =
        branch::count_dirty_files(&git_repo).map_err(|_| "Vérification impossible".to_string())?;
    if dirty == 0 {
        return branch::checkout_branch(repo_path, target_branch);
    }

    let description = sanitize_description(description)?;
    let signature = commit_signature(&git_repo, &workdir)?;
    let index_backup = IndexBackup::capture(&git_repo)?;
    if let Err(e) = create_wip_commit(&git_repo, target_branch, description.as_deref(), &signature)
    {
        index_backup.restore();
        return Err(e);
    }

    branch::checkout_branch(repo_path, target_branch)
}

fn ensure_local_branch_exists(repo: &Repository, branch_name: &str) -> Result<(), String> {
    repo.find_branch(branch_name, BranchType::Local)
        .map(|_| ())
        .map_err(|_| "Branche introuvable".to_string())
}

fn create_wip_commit(
    repo: &Repository,
    target_branch: &str,
    description: Option<&str>,
    signature: &Signature<'_>,
) -> Result<(), String> {
    let mut index = repo.index().map_err(|e| format!("Index : {e}"))?;
    index
        .add_all(["*"], IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("Staging : {e}"))?;
    index.write().map_err(|e| format!("Écriture index : {e}"))?;

    let tree_oid = index.write_tree().map_err(|e| format!("Arbre : {e}"))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| format!("Arbre : {e}"))?;
    let parent = repo
        .head()
        .and_then(|h| h.peel_to_commit())
        .map_err(|e| format!("Commit parent : {e}"))?;
    let msg = build_commit_message(target_branch, description);
    repo.commit(Some("HEAD"), signature, signature, &msg, &tree, &[&parent])
        .map_err(|e| format!("Commit : {e}"))?;
    Ok(())
}

fn build_commit_message(target_branch: &str, description: Option<&str>) -> String {
    let subject = format!("WIP: save changes before switching to {target_branch}");
    match description {
        Some(body) if !body.is_empty() => format!("{subject}\n\n{body}"),
        _ => subject,
    }
}

fn sanitize_description(description: Option<String>) -> Result<Option<String>, String> {
    let Some(description) = description else {
        return Ok(None);
    };
    let normalized = description.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_COMMIT_DESCRIPTION_CHARS
        || trimmed
            .chars()
            .any(|c| c == '\0' || (c.is_control() && c != '\n' && c != '\t'))
    {
        return Err("Description de commit invalide".to_string());
    }
    Ok(Some(trimmed.to_string()))
}

fn commit_signature(repo: &Repository, workdir: &Path) -> Result<Signature<'static>, String> {
    if let Ok(sig) = repo.signature() {
        return Ok(sig);
    }
    signature_from_git_var(workdir).ok_or_else(|| "Identité Git introuvable".to_string())
}

fn signature_from_git_var(workdir: &Path) -> Option<Signature<'static>> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(workdir)
        .args(["var", "GIT_AUTHOR_IDENT"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let ident = String::from_utf8(output.stdout).ok()?;
    let (name, email) = parse_git_ident(&ident)?;
    Signature::now(&name, &email).ok()
}

pub(super) fn parse_git_ident(ident: &str) -> Option<(String, String)> {
    let start = ident.rfind('<')?;
    let end = ident[start..].find('>')? + start;
    let name = ident[..start].trim();
    let email = ident[start + 1..end].trim();
    if name.is_empty() || email.is_empty() {
        return None;
    }
    Some((name.to_string(), email.to_string()))
}

struct IndexBackup {
    index_path: PathBuf,
    backup: Option<tempfile::NamedTempFile>,
}

impl IndexBackup {
    fn capture(repo: &Repository) -> Result<Self, String> {
        let index_path = repo.path().join("index");
        if !index_path.exists() {
            return Ok(Self {
                index_path,
                backup: None,
            });
        }
        let backup = tempfile::NamedTempFile::new_in(repo.path())
            .map_err(|_| "Sauvegarde index impossible".to_string())?;
        std::fs::copy(&index_path, backup.path())
            .map_err(|_| "Sauvegarde index impossible".to_string())?;
        Ok(Self {
            index_path,
            backup: Some(backup),
        })
    }

    fn restore(&self) {
        if let Some(backup) = &self.backup {
            let _ = std::fs::copy(backup.path(), &self.index_path);
        } else {
            let _ = std::fs::remove_file(&self.index_path);
        }
    }
}
