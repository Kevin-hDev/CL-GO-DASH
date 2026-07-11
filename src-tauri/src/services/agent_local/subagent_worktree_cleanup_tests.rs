use super::{subagent_worktree, subagent_worktree_ownership_tests::init_repo_with_commit};
use std::path::Path;
use std::process::Command;

#[tokio::test]
async fn locked_worktree_cleanup_removes_git_metadata() {
    let repo = init_repo_with_commit();
    let child_id = id();
    let execution_id = id();
    let target = subagent_worktree::create_for_execution(repo.path(), &child_id, &execution_id)
        .await
        .expect("create worktree");
    let locked = Command::new("git")
        .arg("-C")
        .arg(repo.path())
        .args(["worktree", "lock"])
        .arg(&target)
        .status()
        .expect("lock worktree");
    assert!(locked.success());

    let result = subagent_worktree::remove(&target.to_string_lossy()).await;
    let listed = worktree_list(repo.path());

    assert!(result.is_ok());
    assert!(!target.exists());
    assert!(
        !listed.contains(target.to_string_lossy().as_ref()),
        "les métadonnées Git du worktree verrouillé subsistent"
    );
}

#[tokio::test]
async fn successful_cleanup_removes_empty_child_directory() {
    let repo = init_repo_with_commit();
    let target = subagent_worktree::create_for_execution(repo.path(), &id(), &id())
        .await
        .expect("create worktree");
    let child_dir = target.parent().expect("child directory").to_path_buf();

    subagent_worktree::remove(&target.to_string_lossy())
        .await
        .expect("remove worktree");

    assert!(!target.exists());
    assert!(!child_dir.exists(), "le répertoire UUID vide subsiste");
}

#[tokio::test]
async fn missing_execution_cleanup_removes_empty_child_directory() {
    let target = subagent_worktree::path_for_execution(&id(), &id()).expect("managed path");
    let child_dir = target.parent().expect("child directory").to_path_buf();
    tokio::fs::create_dir_all(&child_dir)
        .await
        .expect("create empty child directory");

    subagent_worktree::remove(&target.to_string_lossy())
        .await
        .expect("missing worktree is already removed");

    assert!(!child_dir.exists(), "le répertoire UUID vide subsiste après l'échec initial");
}

#[tokio::test]
async fn cleanup_never_removes_a_sibling_execution() {
    let repo = init_repo_with_commit();
    let child_id = id();
    let first = subagent_worktree::create_for_execution(repo.path(), &child_id, &id())
        .await
        .expect("create first worktree");
    let sibling = subagent_worktree::create_for_execution(repo.path(), &child_id, &id())
        .await
        .expect("create sibling worktree");
    let child_dir = first.parent().expect("child directory").to_path_buf();

    subagent_worktree::remove(&first.to_string_lossy())
        .await
        .expect("remove first worktree");

    assert!(child_dir.exists());
    assert!(sibling.exists(), "un autre run a été supprimé");
    subagent_worktree::remove(&sibling.to_string_lossy())
        .await
        .expect("remove sibling worktree");
}

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn worktree_list(repo: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .expect("list worktrees");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("utf8 worktree list")
}
