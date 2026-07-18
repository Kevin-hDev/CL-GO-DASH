use super::{branch, remote};
use git2::{Repository, Signature};
use std::path::Path;

#[test]
fn publishes_branch_and_tracks_ahead_commits() {
    let local = init_repo();
    let bare = tempfile::tempdir().expect("bare dir");
    Repository::init_bare(bare.path()).expect("bare repo");
    let repo = Repository::open(local.path()).expect("open local");
    repo.remote("origin", bare.path().to_str().expect("remote path"))
        .expect("remote");
    drop(repo);

    let initial = remote::status(local.path()).expect("initial status");
    assert!(initial.has_remote);
    assert!(!initial.has_upstream);

    remote::push_current(local.path(), None).expect("first push");
    let published = remote::status(local.path()).expect("published status");
    assert!(published.has_upstream);
    assert_eq!(published.ahead, 0);

    std::fs::write(local.path().join("next.txt"), "next").expect("next file");
    commit_path(local.path(), "next.txt", "next");
    assert_eq!(remote::status(local.path()).expect("ahead status").ahead, 1);

    remote::push_current(local.path(), None).expect("second push");
    assert_eq!(
        remote::status(local.path()).expect("synced status").ahead,
        0
    );
    assert_eq!(branch::get_context(local.path()).branch, "main");
}

#[test]
fn github_ssh_remote_does_not_require_oauth_token() {
    let local = init_repo();
    let repo = Repository::open(local.path()).expect("open local");
    repo.remote("origin", "git@github.com:example/project.git")
        .expect("ssh remote");
    drop(repo);

    assert!(!remote::remote_requires_github_token(local.path()));
    assert!(remote::status(local.path()).expect("ssh status").is_github);
    let repo = Repository::open(local.path()).expect("reopen local");
    repo.remote_set_url("origin", "https://github.com/example/project.git")
        .expect("https remote");
    drop(repo);
    assert!(remote::remote_requires_github_token(local.path()));
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
    commit_path(tmp.path(), "README.md", "init");
    let repo = Repository::open(tmp.path()).expect("reopen repo");
    repo.branch(
        "main",
        &repo.head().unwrap().peel_to_commit().unwrap(),
        false,
    )
    .expect("main branch");
    repo.set_head("refs/heads/main").expect("set main");
    drop(repo);
    tmp
}

fn commit_path(root: &Path, relative: &str, message: &str) {
    let repo = Repository::open(root).expect("open repo");
    let mut index = repo.index().expect("index");
    index.add_path(Path::new(relative)).expect("add path");
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("tree oid");
    let tree = repo.find_tree(tree_oid).expect("tree");
    let signature = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let parents = parent.iter().collect::<Vec<_>>();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )
    .expect("commit");
}
