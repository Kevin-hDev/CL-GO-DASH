//! Tests du parsing `gitdir:` de watcher.rs (PURE).
//!
//! `parse_gitdir_content` parse le contenu d'un fichier `.git` pointant vers
//! un worktree linked (format : `gitdir: /chemin`). C'est la logique critique
//! de résolution du dossier Git réel, extraite en fn PURE pour etre testée
//! sans toucher au système de fichiers.

use super::watcher::{parse_gitdir_content, read_dirty_count, update_dirty_count};
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
    let mut previous = read_dirty_count(tmp.path());
    assert_eq!(previous, Some(0));

    std::fs::write(tmp.path().join("tracked.txt"), "changed\n").expect("modify tracked file");
    let current = read_dirty_count(tmp.path());

    assert_eq!(current, Some(1));
    assert!(update_dirty_count(&mut previous, current));
    assert_eq!(previous, Some(1));

    std::fs::write(tmp.path().join("new.txt"), "new\n").expect("create untracked file");
    let current = read_dirty_count(tmp.path());

    assert_eq!(current, Some(2));
    assert!(update_dirty_count(&mut previous, current));
}

#[test]
fn ignores_unchanged_or_temporarily_unavailable_status() {
    let mut previous = Some(1);

    assert!(!update_dirty_count(&mut previous, Some(1)));
    assert!(!update_dirty_count(&mut previous, None));
    assert_eq!(previous, Some(1));
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
    repo.commit(Some("HEAD"), &signature, &signature, "initial", &tree, &[])
        .expect("commit");
}
