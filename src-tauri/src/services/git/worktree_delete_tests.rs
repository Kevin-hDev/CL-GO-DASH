use super::worktree_delete;
use git2::{Repository, Signature};
use std::path::Path;
use std::process::Command;

#[tokio::test]
async fn previews_and_preserves_dirty_worktree_before_removal() {
    let repo_dir = init_repo();
    let worktree_root = tempfile::tempdir().expect("worktree parent");
    let worktree_path = worktree_root.path().join("feature-worktree");
    let status = Command::new("git")
        .args(["-C"])
        .arg(repo_dir.path())
        .args(["worktree", "add", "-b", "feature"])
        .arg(&worktree_path)
        .status()
        .expect("worktree add");
    assert!(status.success());
    std::fs::write(worktree_path.join("dirty.txt"), "keep").expect("dirty file");

    let preview = worktree_delete::preview(repo_dir.path(), &worktree_path)
        .await
        .expect("preview");
    assert_eq!(preview.branch, "feature");
    assert!(preview
        .dirty_files
        .iter()
        .any(|file| file.path == "dirty.txt"));

    worktree_delete::preserve_and_remove(
        repo_dir.path(),
        &worktree_path,
        Some("Conserver le travail".to_string()),
    )
    .await
    .expect("preserve and remove");

    assert!(!worktree_path.exists());
    let repo = Repository::open(repo_dir.path()).expect("repo");
    assert!(repo.find_branch("feature", git2::BranchType::Local).is_ok());
}

fn init_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    let mut config = repo.config().expect("config");
    config.set_str("user.name", "CL-GO Test").expect("name");
    config
        .set_str("user.email", "test@example.com")
        .expect("email");
    std::fs::write(tmp.path().join("README.md"), "init").expect("readme");
    let mut index = repo.index().expect("index");
    index.add_path(Path::new("README.md")).expect("add");
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("tree oid");
    let tree = repo.find_tree(tree_oid).expect("tree");
    let signature = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &signature, &signature, "init", &tree, &[])
        .expect("commit");
    drop(tree);
    drop(repo);
    tmp
}
