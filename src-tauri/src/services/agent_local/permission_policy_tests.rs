use crate::services::agent_local::permission_policy::{is_data_dir_write, uses_auto_bypass};
use serde_json::json;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    crate::services::paths::data_dir()
}

#[test]
fn auto_modes_use_bypass_policy() {
    assert!(uses_auto_bypass("auto"));
    assert!(uses_auto_bypass("subagent"));
    assert!(!uses_auto_bypass("manual"));
    assert!(!uses_auto_bypass("chat"));
}

#[test]
fn absolute_write_file_in_data_dir_needs_permission() {
    let path = data_dir().join("agent-settings.json");
    let args = json!({ "path": path.to_string_lossy() });
    assert!(is_data_dir_write(
        "write_file",
        &args,
        &PathBuf::from("/tmp")
    ));
}

#[test]
fn relative_write_file_in_data_dir_needs_permission() {
    let args = json!({ "path": "agent-settings.json" });
    assert!(is_data_dir_write("write_file", &args, &data_dir()));
}

#[test]
fn write_file_outside_data_dir_does_not_need_permission() {
    let args = json!({ "path": "src/main.rs" });
    assert!(!is_data_dir_write(
        "write_file",
        &args,
        &PathBuf::from("/tmp/project")
    ));
}

#[test]
fn write_file_inside_subagent_worktree_does_not_need_permission() {
    let path = data_dir().join("subagent-worktrees/child/file.md");
    let args = json!({ "path": path.to_string_lossy() });
    assert!(!is_data_dir_write(
        "write_file",
        &args,
        &PathBuf::from("/tmp")
    ));
}

#[test]
fn edit_file_in_data_dir_needs_permission() {
    let args = json!({ "path": data_dir().join("config.json").to_string_lossy() });
    assert!(is_data_dir_write(
        "edit_file",
        &args,
        &PathBuf::from("/tmp")
    ));
}

#[test]
fn process_image_output_in_data_dir_needs_permission() {
    let args = json!({
        "input_path": "/tmp/input.png",
        "output_path": data_dir().join("out.png").to_string_lossy(),
    });
    assert!(is_data_dir_write(
        "process_image",
        &args,
        &PathBuf::from("/tmp")
    ));
}

#[test]
fn web_fetch_is_not_a_data_dir_write() {
    let args = json!({ "url": "https://example.com" });
    assert!(!is_data_dir_write("web_fetch", &args, &data_dir()));
}

#[test]
fn unsafe_bash_from_data_dir_needs_permission() {
    let args = json!({ "command": "touch generated.txt" });
    assert!(is_data_dir_write("bash", &args, &data_dir()));
}

#[test]
fn unsafe_bash_from_subagent_worktree_does_not_need_permission() {
    let args = json!({ "command": "touch generated.txt" });
    let working_dir = data_dir().join("subagent-worktrees/child");
    assert!(!is_data_dir_write("bash", &args, &working_dir));
}

#[test]
fn safe_bash_from_data_dir_does_not_need_permission() {
    let args = json!({ "command": "ls -la" });
    assert!(!is_data_dir_write("bash", &args, &data_dir()));
}

#[test]
fn unsafe_bash_mentions_subagent_worktree_does_not_need_permission() {
    let target = data_dir().join("subagent-worktrees/child/generated.txt");
    let args = json!({ "command": format!("touch {}", target.display()) });
    assert!(!is_data_dir_write("bash", &args, &PathBuf::from("/tmp")));
}

#[test]
fn unsafe_bash_mentions_data_dir_needs_permission() {
    let command = format!("touch {}/generated.txt", data_dir().display());
    let args = json!({ "command": command });
    assert!(is_data_dir_write("bash", &args, &PathBuf::from("/tmp")));
}

#[test]
fn unsafe_bash_outside_data_dir_does_not_need_permission() {
    let args = json!({ "command": "touch generated.txt" });
    assert!(!is_data_dir_write("bash", &args, &PathBuf::from("/tmp")));
}
