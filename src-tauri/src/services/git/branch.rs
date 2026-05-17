use git2::{BranchType, Repository, StatusOptions};
use serde::Serialize;
use std::path::Path;

const MAX_BRANCHES: usize = 500;

#[derive(Debug, Clone, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub dirty_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitContext {
    pub branch: String,
    pub is_detached: bool,
    pub dirty_count: usize,
    pub is_git_repo: bool,
}

pub fn list_branches(repo_path: &Path) -> Result<Vec<BranchInfo>, String> {
    let repo =
        Repository::open(repo_path).map_err(|e| format!("Impossible d'ouvrir le dépôt : {e}"))?;

    let current_name = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from));

    let dirty_count = count_dirty_files(&repo).unwrap_or(0);
    let mut branches = Vec::new();

    for branch_result in repo
        .branches(Some(BranchType::Local))
        .map_err(|e| format!("Lecture des branches : {e}"))?
    {
        if branches.len() >= MAX_BRANCHES {
            break;
        }
        let (branch, _) = branch_result.map_err(|e| format!("Lecture branche : {e}"))?;
        let name = branch
            .name()
            .map_err(|e| format!("Nom de branche : {e}"))?
            .unwrap_or("?")
            .to_string();
        let is_current = current_name.as_deref() == Some(name.as_str());

        branches.push(BranchInfo {
            name,
            is_current,
            is_remote: false,
            dirty_count: if is_current { dirty_count } else { 0 },
        });
    }

    branches.sort_by(|a, b| b.is_current.cmp(&a.is_current).then(a.name.cmp(&b.name)));

    Ok(branches)
}

fn count_dirty_files(repo: &Repository) -> Result<usize, git2::Error> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);
    let statuses = repo.statuses(Some(&mut opts))?;
    Ok(statuses.len())
}

pub fn get_context(repo_path: &Path) -> GitContext {
    let repo = match Repository::open(repo_path) {
        Ok(r) => r,
        Err(_) => {
            return GitContext {
                branch: String::new(),
                is_detached: false,
                dirty_count: 0,
                is_git_repo: false,
            }
        }
    };

    let is_detached = repo.head_detached().unwrap_or(false);
    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "HEAD".to_string());
    let dirty_count = count_dirty_files(&repo).unwrap_or(0);

    GitContext {
        branch,
        is_detached,
        dirty_count,
        is_git_repo: true,
    }
}

fn validate_branch_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.contains("..")
        || name.contains('\0')
        || name.contains('\\')
        || name.contains(':')
        || name.contains('~')
        || name.contains('^')
        || name.contains('?')
        || name.contains('*')
        || name.contains('[')
        || name.contains("@{")
        || name.contains("//")
        || name.starts_with('/')
        || name.ends_with('/')
        || name.ends_with(".lock")
        || name.as_bytes().iter().any(|&b| b <= 0x20 || b == 0x7f)
    {
        return Err("Nom de branche invalide".to_string());
    }
    for segment in name.split('/') {
        if segment.is_empty() || segment.starts_with('.') || segment.ends_with('.') {
            return Err("Nom de branche invalide".to_string());
        }
    }
    Ok(())
}

pub fn checkout_branch(repo_path: &Path, branch_name: &str) -> Result<(), String> {
    validate_branch_name(branch_name)?;

    let repo =
        Repository::open(repo_path).map_err(|e| format!("Impossible d'ouvrir le dépôt : {e}"))?;

    let dirty = count_dirty_files(&repo).map_err(|_| "Vérification impossible".to_string())?;
    if dirty > 0 {
        return Err(format!("DIRTY:{dirty}"));
    }

    let (object, reference) = repo
        .revparse_ext(&format!("refs/heads/{branch_name}"))
        .map_err(|e| format!("Branche introuvable : {e}"))?;

    repo.checkout_tree(&object, None)
        .map_err(|e| format!("Checkout impossible : {e}"))?;

    if let Some(refname) = reference.as_ref().and_then(|r| r.name()) {
        repo.set_head(refname)
            .map_err(|e| format!("Mise à jour HEAD : {e}"))?;
    }

    Ok(())
}

pub fn commit_all_and_checkout(repo_path: &Path, target_branch: &str) -> Result<(), String> {
    validate_branch_name(target_branch)?;

    let repo =
        Repository::open(repo_path).map_err(|e| format!("Impossible d'ouvrir le dépôt : {e}"))?;

    let mut index = repo.index().map_err(|e| format!("Index : {e}"))?;
    index
        .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("Staging : {e}"))?;
    index.write().map_err(|e| format!("Écriture index : {e}"))?;

    let tree_oid = index.write_tree().map_err(|e| format!("Arbre : {e}"))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| format!("Arbre : {e}"))?;

    let head = repo.head().map_err(|e| format!("HEAD : {e}"))?;
    let parent = head
        .peel_to_commit()
        .map_err(|e| format!("Commit parent : {e}"))?;

    let sig = repo
        .signature()
        .map_err(|e| format!("Signature git : {e}"))?;
    let msg = format!("WIP: save changes before switching to {target_branch}");

    repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&parent])
        .map_err(|e| format!("Commit : {e}"))?;

    checkout_branch(repo_path, target_branch)
}

pub fn create_branch(repo_path: &Path, branch_name: &str) -> Result<(), String> {
    validate_branch_name(branch_name)?;

    if branch_name.starts_with('-') || branch_name.contains(' ') {
        return Err("Nom de branche invalide".to_string());
    }
    if branch_name.len() > 100 {
        return Err("Nom de branche trop long".to_string());
    }

    let repo =
        Repository::open(repo_path).map_err(|e| format!("Impossible d'ouvrir le dépôt : {e}"))?;

    let head_commit = repo
        .head()
        .map_err(|e| format!("HEAD invalide : {e}"))?
        .peel_to_commit()
        .map_err(|e| format!("Commit HEAD : {e}"))?;

    repo.branch(branch_name, &head_commit, false)
        .map_err(|e| format!("Création branche : {e}"))?;

    repo.set_head(&format!("refs/heads/{branch_name}"))
        .map_err(|e| format!("Mise à jour HEAD : {e}"))?;

    Ok(())
}
