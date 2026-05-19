use git2::{BranchType, IndexAddOption, Repository, Signature};
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{branch, repo};

pub fn commit_all_and_checkout(repo_path: &Path, target_branch: &str) -> Result<(), String> {
    branch::validate_branch_name(target_branch)?;
    let git_repo = repo::open(repo_path)?;
    let workdir = repo::workdir(&git_repo)?;
    ensure_local_branch_exists(&git_repo, target_branch)?;

    let dirty = branch::count_dirty_files(&git_repo)
        .map_err(|_| "Vérification impossible".to_string())?;
    if dirty == 0 {
        return branch::checkout_branch(repo_path, target_branch);
    }

    let signature = commit_signature(&git_repo, &workdir)?;
    let index_backup = IndexBackup::capture(&git_repo)?;
    if let Err(e) = create_wip_commit(&git_repo, target_branch, &signature) {
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
    signature: &Signature<'_>,
) -> Result<(), String> {
    let mut index = repo.index().map_err(|e| format!("Index : {e}"))?;
    index
        .add_all(["*"], IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("Staging : {e}"))?;
    index.write().map_err(|e| format!("Écriture index : {e}"))?;

    let tree_oid = index.write_tree().map_err(|e| format!("Arbre : {e}"))?;
    let tree = repo.find_tree(tree_oid).map_err(|e| format!("Arbre : {e}"))?;
    let parent = repo
        .head()
        .and_then(|h| h.peel_to_commit())
        .map_err(|e| format!("Commit parent : {e}"))?;
    let msg = format!("WIP: save changes before switching to {target_branch}");
    repo.commit(Some("HEAD"), signature, signature, &msg, &tree, &[&parent])
        .map_err(|e| format!("Commit : {e}"))?;
    Ok(())
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

fn parse_git_ident(ident: &str) -> Option<(String, String)> {
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
            return Ok(Self { index_path, backup: None });
        }
        let backup = tempfile::NamedTempFile::new_in(repo.path())
            .map_err(|_| "Sauvegarde index impossible".to_string())?;
        std::fs::copy(&index_path, backup.path())
            .map_err(|_| "Sauvegarde index impossible".to_string())?;
        Ok(Self { index_path, backup: Some(backup) })
    }

    fn restore(&self) {
        if let Some(backup) = &self.backup {
            let _ = std::fs::copy(backup.path(), &self.index_path);
        } else {
            let _ = std::fs::remove_file(&self.index_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{commit_all_and_checkout, parse_git_ident};
    use crate::services::git::branch;
    use git2::{Repository, Signature};
    use std::path::Path;

    #[test]
    fn parses_git_author_ident() {
        let parsed = parse_git_ident("Kevin Huynh <kevin@example.com> 1779207754 +0200");
        assert_eq!(
            parsed,
            Some(("Kevin Huynh".to_string(), "kevin@example.com".to_string()))
        );
    }

    #[test]
    fn commits_deletions_then_switches_branch() {
        let tmp = init_repo();
        let repo = Repository::open(tmp.path()).expect("open repo");
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("target", &head, false).expect("create target");
        drop(head);
        drop(repo);

        std::fs::remove_file(tmp.path().join("remove.txt")).expect("delete file");
        std::fs::write(tmp.path().join("new.txt"), "new").expect("new file");

        commit_all_and_checkout(tmp.path(), "target").expect("commit and checkout");

        let ctx = branch::get_context(tmp.path());
        assert_eq!(ctx.branch, "target");
        assert_eq!(ctx.dirty_count, 0);
    }

    fn init_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("temp repo");
        let repo = Repository::init(tmp.path()).expect("init repo");
        let mut cfg = repo.config().expect("config");
        cfg.set_str("user.name", "CL-GO Test").expect("name");
        cfg.set_str("user.email", "test@example.com").expect("email");
        std::fs::write(tmp.path().join("keep.txt"), "keep").expect("keep");
        std::fs::write(tmp.path().join("remove.txt"), "remove").expect("remove");
        commit_paths(&repo, tmp.path(), &["keep.txt", "remove.txt"]);
        drop(repo);
        tmp
    }

    fn commit_paths(repo: &Repository, root: &Path, paths: &[&str]) {
        let mut index = repo.index().expect("index");
        for path in paths {
            index.add_path(Path::new(path)).expect("add path");
        }
        index.write().expect("write index");
        let tree_oid = index.write_tree().expect("tree");
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let sig = Signature::now("CL-GO Test", "test@example.com").expect("signature");
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .expect("commit");
        assert!(root.join("keep.txt").exists());
    }
}
