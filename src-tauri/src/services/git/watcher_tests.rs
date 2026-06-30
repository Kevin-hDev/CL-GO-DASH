//! Tests du parsing `gitdir:` de watcher.rs (PURE).
//!
//! `parse_gitdir_content` parse le contenu d'un fichier `.git` pointant vers
//! un worktree linked (format : `gitdir: /chemin`). C'est la logique critique
//! de résolution du dossier Git réel, extraite en fn PURE pour etre testée
//! sans toucher au système de fichiers.

use super::watcher::parse_gitdir_content;
use std::path::PathBuf;

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
