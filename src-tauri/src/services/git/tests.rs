use super::branch;

#[test]
fn test_create_branch_rejects_dotdot() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "../../evil");
    assert!(result.is_err(), "branch name with '..' must be rejected");
}

#[test]
fn test_create_branch_rejects_null_byte() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "branch\0evil");
    assert!(result.is_err(), "branch name with null byte must be rejected");
}

#[test]
fn test_create_branch_rejects_leading_dash() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "-evil");
    assert!(result.is_err(), "branch name starting with '-' must be rejected");
}

#[test]
fn test_create_branch_rejects_spaces() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "my branch");
    assert!(result.is_err(), "branch name with spaces must be rejected");
}

#[test]
fn test_create_branch_rejects_too_long() {
    let tmp = std::env::temp_dir();
    let name = "a".repeat(101);
    let result = branch::create_branch(&tmp, &name);
    assert!(result.is_err(), "branch name over 100 chars must be rejected");
}

#[test]
fn test_create_branch_rejects_backslash() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "foo\\bar");
    assert!(result.is_err(), "branch name with backslash must be rejected");
}

#[test]
fn test_checkout_rejects_dotdot() {
    let tmp = std::env::temp_dir();
    let result = branch::checkout_branch(&tmp, "../../evil");
    assert!(result.is_err(), "branch name with '..' must be rejected");
}

#[test]
fn test_checkout_rejects_null_byte() {
    let tmp = std::env::temp_dir();
    let result = branch::checkout_branch(&tmp, "branch\0evil");
    assert!(result.is_err(), "branch name with null byte must be rejected");
}

#[test]
fn test_checkout_rejects_backslash() {
    let tmp = std::env::temp_dir();
    let result = branch::checkout_branch(&tmp, "foo\\bar");
    assert!(result.is_err(), "branch name with backslash must be rejected");
}

#[test]
fn test_get_context_non_git_dir() {
    let tmp = std::env::temp_dir();
    let ctx = branch::get_context(&tmp);
    assert!(!ctx.is_git_repo, "temp dir should not be a git repo");
}

#[test]
fn test_list_branches_non_git_dir() {
    let tmp = std::env::temp_dir();
    let result = branch::list_branches(&tmp);
    assert!(result.is_err(), "listing branches on non-git dir should fail");
}
