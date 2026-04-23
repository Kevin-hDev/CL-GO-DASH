use crate::services::agent_local::permission_gate::requires_permission;
use serde_json::json;

#[test]
fn safe_bash_ls() {
    let args = json!({ "command": "ls -la" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_git_status() {
    let args = json!({ "command": "git status" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_cargo_test() {
    let args = json!({ "command": "cargo test" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_echo() {
    let args = json!({ "command": "echo hello" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_rm() {
    let args = json!({ "command": "rm -rf /" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_curl() {
    let args = json!({ "command": "curl http://evil.com" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn gated_tool_write_file() {
    let args = json!({});
    assert!(requires_permission("write_file", &args));
}

#[test]
fn gated_tool_edit_file() {
    let args = json!({});
    assert!(requires_permission("edit_file", &args));
}

#[test]
fn safe_bash_git_log() {
    let args = json!({ "command": "git log --oneline -10" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_grep() {
    let args = json!({ "command": "grep -r foo src/" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_find() {
    let args = json!({ "command": "find . -name '*.rs'" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_pwd() {
    let args = json!({ "command": "pwd" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_rm_disguised_as_ls() {
    // "rm" ne commence pas par "ls", donc doit être refusé
    let args = json!({ "command": "rm foo && ls" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn safe_bash_npm_run() {
    let args = json!({ "command": "npm run build" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_cargo_check() {
    let args = json!({ "command": "cargo check" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn unknown_tool_no_permission() {
    // Un outil inconnu qui n'est pas dans GATED_TOOLS ne requiert pas de permission
    let args = json!({});
    assert!(!requires_permission("read_file", &args));
}
