use super::{branch, repo as git_repo};
use git2::BranchType;
use std::path::Path;

pub fn delete_branch(repo_path: &Path, branch_name: &str) -> Result<(), String> {
    branch::validate_branch_name(branch_name).map_err(|e| e.to_string())?;
    let repo = git_repo::open(repo_path)?;
    let current = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().map(str::to_string));
    if current.as_deref() == Some(branch_name) {
        return Err("Branche active".into());
    }
    let mut branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|_| "Branche introuvable".to_string())?;
    branch.delete().map_err(|_| "Suppression impossible".to_string())
}

pub fn branch_exists(repo_path: &Path, branch_name: &str) -> Result<bool, String> {
    branch::validate_branch_name(branch_name).map_err(|e| e.to_string())?;
    let repo = git_repo::open(repo_path)?;
    let exists = repo.find_branch(branch_name, BranchType::Local).is_ok();
    Ok(exists)
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature};

    fn init_repo_with_commit() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("temp repo");
        let repo = Repository::init(tmp.path()).expect("init repo");
        std::fs::write(tmp.path().join("README.md"), "init").expect("write file");
        let mut index = repo.index().expect("index");
        index.add_path(std::path::Path::new("README.md")).expect("add");
        index.write().expect("write index");
        let tree_id = index.write_tree().expect("tree");
        let tree = repo.find_tree(tree_id).expect("find tree");
        let sig = Signature::now("CL-GO Test", "test@example.com").expect("signature");
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .expect("commit");
        tmp
    }

    #[test]
    fn delete_branch_removes_local_branch() {
        let tmp = init_repo_with_commit();
        branch::create_branch(tmp.path(), "clone-11111111").expect("create branch");
        branch::checkout_branch(tmp.path(), "master").expect("checkout master");
        assert!(branch_exists(tmp.path(), "clone-11111111").expect("exists"));
        delete_branch(tmp.path(), "clone-11111111").expect("delete branch");
        assert!(!branch_exists(tmp.path(), "clone-11111111").expect("exists"));
    }

    #[test]
    fn delete_branch_refuses_current_branch() {
        let tmp = init_repo_with_commit();
        branch::create_branch(tmp.path(), "clone-11111111").expect("create branch");
        let result = delete_branch(tmp.path(), "clone-11111111");
        assert!(result.is_err());
    }
}
