//! Tests du parsing `gitdir:` de watcher.rs (PURE).
//!
//! `parse_gitdir_content` parse le contenu d'un fichier `.git` pointant vers
//! un worktree linked (format : `gitdir: /chemin`). C'est la logique critique
//! de résolution du dossier Git réel, extraite en fn PURE pour etre testée
//! sans toucher au système de fichiers.

use super::watcher::{parse_gitdir_content, read_watch_state, update_watch_state};
use git2::{Repository, Signature};
use std::path::{Path, PathBuf};

#[test]
fn parses_absolute_gitdir() {
    let content = "gitdir: /Users/dev/proj/.git/worktrees/feature\n";
    let result = parse_gitdir_content(content);
    assert_eq!(
        result,
        Some(PathBuf::from("/Users/dev/proj/.git/worktrees/feature"))
    );
}

#[test]
fn parses_relative_gitdir() {
    let content = "gitdir: ../.git/worktrees/feature";
    let result = parse_gitdir_content(content);
    assert_eq!(result, Some(PathBuf::from("../.git/worktrees/feature")));
}

#[test]
fn trims_whitespace() {
    let content = "  gitdir: /abs/path  \n";
    let result = parse_gitdir_content(content);
    assert_eq!(result, Some(PathBuf::from("/abs/path")));
}

#[test]
fn returns_none_without_prefix() {
    assert!(parse_gitdir_content("/just/a/path").is_none());
    assert!(parse_gitdir_content("not a gitdir file").is_none());
}

#[test]
fn returns_none_for_empty_content() {
    assert!(parse_gitdir_content("").is_none());
}

#[test]
fn returns_none_for_prefix_only() {
    // "gitdir: " sans chemin après → None.
    assert!(parse_gitdir_content("gitdir: ").is_none());
    assert!(parse_gitdir_content("gitdir:").is_none());
}

#[test]
fn parses_unix_worktree_path() {
    let content = "gitdir: /home/user/repo/.git/worktrees/feat-x";
    let result = parse_gitdir_content(content).unwrap();
    assert!(result.starts_with("/home/user/repo"));
}

#[test]
fn handles_trailing_newline_only() {
    // Cas réel : fichier .git avec juste un retour à la ligne final.
    let content = "gitdir: /path/to/.git/worktrees/main\n";
    assert_eq!(
        parse_gitdir_content(content),
        Some(PathBuf::from("/path/to/.git/worktrees/main"))
    );
}

#[test]
fn detects_a_new_change_after_a_clean_commit() {
    let tmp = init_repo();
    let mut previous = read_watch_state(tmp.path());
    assert_eq!(dirty_count(&previous), Some(0));

    std::fs::write(tmp.path().join("tracked.txt"), "changed\n").expect("modify tracked file");
    let current = read_watch_state(tmp.path());

    assert_eq!(dirty_count(&current), Some(1));
    assert!(update_watch_state(&mut previous, current));
    assert_eq!(dirty_count(&previous), Some(1));

    std::fs::write(tmp.path().join("new.txt"), "new\n").expect("create untracked file");
    let current = read_watch_state(tmp.path());

    assert_eq!(dirty_count(&current), Some(2));
    assert!(update_watch_state(&mut previous, current));
}

#[test]
fn detects_an_edit_immediately_after_commit_with_the_same_dirty_count() {
    let tmp = init_repo();
    let repo = Repository::open(tmp.path()).expect("open repo");
    std::fs::write(tmp.path().join("tracked.txt"), "first edit\n").expect("first edit");
    let mut previous = read_watch_state(tmp.path());
    assert_eq!(dirty_count(&previous), Some(1));

    commit_all(&repo);
    std::fs::write(tmp.path().join("tracked.txt"), "second edit\n").expect("second edit");
    let current = read_watch_state(tmp.path());

    assert_eq!(dirty_count(&current), Some(1));
    assert_ne!(head_oid(&previous), head_oid(&current));
    assert!(update_watch_state(&mut previous, current));
}

#[test]
fn detects_repeated_edits_while_the_file_is_already_dirty() {
    let tmp = init_repo();
    std::fs::write(tmp.path().join("tracked.txt"), "first edit\n").expect("first edit");
    let mut previous = read_watch_state(tmp.path());

    std::fs::write(tmp.path().join("tracked.txt"), "second edit with another size\n")
        .expect("second edit");
    let current = read_watch_state(tmp.path());

    assert_eq!(dirty_count(&previous), Some(1));
    assert_eq!(dirty_count(&current), Some(1));
    assert!(update_watch_state(&mut previous, current));
}

#[test]
fn ignores_unchanged_or_temporarily_unavailable_status() {
    let tmp = init_repo();
    let current = read_watch_state(tmp.path());
    let mut previous = current.clone();

    assert!(!update_watch_state(&mut previous, current));
    assert!(!update_watch_state(&mut previous, None));
    assert_eq!(dirty_count(&previous), Some(0));
}

#[test]
fn detects_a_deleted_tracked_file() {
    let tmp = init_repo();
    let mut previous = read_watch_state(tmp.path());

    std::fs::remove_file(tmp.path().join("tracked.txt")).expect("delete tracked file");
    let current = read_watch_state(tmp.path());

    assert_eq!(dirty_count(&current), Some(1));
    assert!(update_watch_state(&mut previous, current));
}

#[test]
fn detects_a_branch_switch_when_both_branches_share_the_same_commit() {
    let tmp = init_repo();
    let repo = Repository::open(tmp.path()).expect("open repo");
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch("feature", &head, false).expect("create branch");
    let mut previous = read_watch_state(tmp.path());

    repo.set_head("refs/heads/feature").expect("switch head");
    let current = read_watch_state(tmp.path());

    assert_eq!(head_oid(&previous), head_oid(&current));
    assert!(update_watch_state(&mut previous, current));
}

fn dirty_count(state: &Option<super::watcher::GitWatchState>) -> Option<usize> {
    state.as_ref().map(|state| state.dirty_count)
}

fn head_oid(state: &Option<super::watcher::GitWatchState>) -> Option<git2::Oid> {
    state.as_ref().and_then(|state| state.head_oid)
}

fn init_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    std::fs::write(tmp.path().join("tracked.txt"), "initial\n").expect("write tracked file");
    commit_all(&repo);
    tmp
}

fn commit_all(repo: &Repository) {
    let mut index = repo.index().expect("index");
    index
        .add_path(Path::new("tracked.txt"))
        .expect("stage file");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("write tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    let signature = Signature::now("Test", "test@example.com").expect("signature");
    let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let parents = parent.iter().collect::<Vec<_>>();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "commit",
        &tree,
        &parents,
    )
    .expect("commit");
}
