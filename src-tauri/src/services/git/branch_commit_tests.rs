use super::{branch, branch_commit};
use git2::{BranchType, Repository, Signature};
use std::path::Path;

#[test]
fn parses_git_author_ident() {
    let parsed = branch_commit::parse_git_ident("Kevin Huynh <kevin@example.com> 1779207754 +0200");
    assert_eq!(
        parsed,
        Some(("Kevin Huynh".to_string(), "kevin@example.com".to_string()))
    );
}

#[test]
fn commits_deletions_then_switches_branch() {
    let tmp = init_repo();
    create_branch(&tmp, "target");
    std::fs::remove_file(tmp.path().join("remove.txt")).expect("delete file");
    std::fs::write(tmp.path().join("new.txt"), "new").expect("new file");

    branch_commit::commit_all_and_checkout(tmp.path(), "target", None)
        .expect("commit and checkout");

    let ctx = branch::get_context(tmp.path());
    assert_eq!(ctx.branch, "target");
    assert_eq!(ctx.dirty_count, 0);
}

#[test]
fn appends_optional_description_to_wip_commit() {
    let tmp = init_repo();
    create_branch(&tmp, "target");
    let source_branch = current_branch_name(tmp.path());
    std::fs::write(tmp.path().join("keep.txt"), "changed").expect("change file");

    branch_commit::commit_all_and_checkout(
        tmp.path(),
        "target",
        Some("Résumé utilisateur\navec détails".to_string()),
    )
    .expect("commit and checkout");

    let repo = Repository::open(tmp.path()).expect("open repo");
    let source_ref = repo
        .find_branch(&source_branch, BranchType::Local)
        .expect("source branch");
    let message = source_ref
        .get()
        .peel_to_commit()
        .unwrap()
        .message()
        .unwrap()
        .to_string();
    assert!(message.contains("WIP: save changes before switching to target"));
    assert!(message.contains("\n\nRésumé utilisateur\navec détails"));
}

fn current_branch_name(root: &Path) -> String {
    Repository::open(root)
        .expect("open repo")
        .head()
        .expect("head")
        .shorthand()
        .expect("branch name")
        .to_string()
}

fn init_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    let mut cfg = repo.config().expect("config");
    cfg.set_str("user.name", "CL-GO Test").expect("name");
    cfg.set_str("user.email", "test@example.com")
        .expect("email");
    std::fs::write(tmp.path().join("keep.txt"), "keep").expect("keep");
    std::fs::write(tmp.path().join("remove.txt"), "remove").expect("remove");
    commit_paths(&repo, tmp.path(), &["keep.txt", "remove.txt"]);
    drop(repo);
    tmp
}

fn create_branch(tmp: &tempfile::TempDir, name: &str) {
    let repo = Repository::open(tmp.path()).expect("open repo");
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch(name, &head, false).expect("create branch");
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

// --- sanitize_description (PURE) — sécurité injection commit ----------------

#[test]
fn sanitize_description_none_stays_none() {
    assert_eq!(branch_commit::sanitize_description(None), Ok(None));
}

#[test]
fn sanitize_description_empty_becomes_none() {
    assert_eq!(
        branch_commit::sanitize_description(Some("   ".to_string())),
        Ok(None)
    );
}

#[test]
fn sanitize_description_normalizes_crlf() {
    // \r\n et \r sont convertis en \n (évite double-escaping dans le commit).
    let result = branch_commit::sanitize_description(Some("a\r\nb\rc".to_string()))
        .unwrap()
        .unwrap();
    assert_eq!(result, "a\nb\nc");
}

#[test]
fn sanitize_description_rejects_null_byte() {
    assert!(branch_commit::sanitize_description(Some("abc\0def".to_string())).is_err());
}

#[test]
fn sanitize_description_rejects_control_chars() {
    // 0x01, 0x1F (sauf \n et \t qui sont autorisés).
    assert!(branch_commit::sanitize_description(Some("bad\x01".to_string())).is_err());
    assert!(branch_commit::sanitize_description(Some("bad\x1F".to_string())).is_err());
}

#[test]
fn sanitize_description_allows_newline_and_tab() {
    let result = branch_commit::sanitize_description(Some("line1\n\tindented".to_string()))
        .unwrap()
        .unwrap();
    assert_eq!(result, "line1\n\tindented");
}

#[test]
fn sanitize_description_rejects_overlong() {
    let too_long = "a".repeat(2001); // MAX_COMMIT_DESCRIPTION_CHARS = 2000
    assert!(branch_commit::sanitize_description(Some(too_long)).is_err());
}

#[test]
fn sanitize_description_accepts_at_max_length() {
    let exact = "a".repeat(2000);
    let result = branch_commit::sanitize_description(Some(exact))
        .unwrap()
        .unwrap();
    assert_eq!(result.chars().count(), 2000);
}

#[test]
fn sanitize_description_trims_whitespace() {
    let result = branch_commit::sanitize_description(Some("  hello  ".to_string()))
        .unwrap()
        .unwrap();
    assert_eq!(result, "hello");
}

// --- build_commit_message (PURE) --------------------------------------------

#[test]
fn build_commit_message_without_description() {
    let msg = branch_commit::build_commit_message("feature", None);
    assert_eq!(msg, "WIP: save changes before switching to feature");
}

#[test]
fn build_commit_message_with_description() {
    let msg = branch_commit::build_commit_message("feature", Some("détails du commit"));
    assert!(msg.contains("WIP: save changes before switching to feature"));
    assert!(msg.contains("\n\ndétails du commit"));
}

#[test]
fn build_commit_message_with_empty_description_omits_body() {
    // Description vide → on garde juste le subject (pas de body).
    let msg = branch_commit::build_commit_message("dev", Some(""));
    assert_eq!(msg, "WIP: save changes before switching to dev");
}
