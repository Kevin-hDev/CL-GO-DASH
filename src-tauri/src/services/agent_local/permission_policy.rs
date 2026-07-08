use crate::services::agent_local::permission_gate::{self, PermissionDecision};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use serde_json::Value;
use std::path::{Component, Path, PathBuf};
use tokio_util::sync::CancellationToken;

const DATA_WRITE_TOOLS: &[&str] = &[
    "write_file",
    "edit_file",
    "write_spreadsheet",
    "write_document",
    "process_image",
];

pub fn uses_auto_bypass(mode: &str) -> bool {
    matches!(mode, "auto" | "subagent")
}

pub fn is_data_dir_write(tool_name: &str, args: &Value, working_dir: &Path) -> bool {
    if tool_name == "bash" {
        return is_bash_data_dir_write(args, working_dir);
    }
    if !DATA_WRITE_TOOLS.contains(&tool_name) {
        return false;
    }

    let path_key = if tool_name == "process_image" {
        "output_path"
    } else {
        "path"
    };
    let Some(path_str) = args[path_key].as_str().filter(|p| !p.trim().is_empty()) else {
        return false;
    };
    path_is_inside_protected_data_dir(&resolve_path(path_str, working_dir))
}

pub async fn check_data_dir_write(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &Value,
    working_dir: &Path,
    cancel: CancellationToken,
) -> bool {
    if !is_data_dir_write(name, args, working_dir) {
        permission_gate::log_diagnostic("data_dir_check_clear", Some(name), Some("not_protected"));
        return true;
    }
    permission_gate::log_diagnostic(
        "data_dir_write_gate",
        Some(name),
        Some("protected_data_dir"),
    );
    match permission_gate::request(on_event, name, args, cancel).await {
        PermissionDecision::Allow | PermissionDecision::AllowSession => true,
        PermissionDecision::Deny => false,
    }
}

fn is_bash_data_dir_write(args: &Value, working_dir: &Path) -> bool {
    if !permission_gate::requires_permission("bash", args) {
        return false;
    }
    let Some(command) = args["command"].as_str() else {
        return false;
    };
    path_is_inside_protected_data_dir(working_dir) || command_mentions_protected_data_dir(command)
}

fn command_mentions_protected_data_dir(command: &str) -> bool {
    let data_dir = crate::services::paths::data_dir();
    let data_dir = normalize_path(&data_dir);
    let data_path = data_dir.to_string_lossy();
    let allowed_root = subagent_worktrees_dir();
    let allowed_root = normalize_path(&allowed_root);
    let allowed_path = allowed_root.to_string_lossy();
    let without_allowed = command
        .replace(allowed_path.as_ref(), "")
        .replace(".local/share/cl-go-dash/subagent-worktrees", "");
    without_allowed.contains(data_path.as_ref())
        || without_allowed.contains(".local/share/cl-go-dash")
}

fn path_is_inside_protected_data_dir(path: &Path) -> bool {
    let path = canonical_or_normalized(path);
    let data_dir = canonical_or_normalized(&crate::services::paths::data_dir());
    let subagent_worktrees = canonical_or_normalized(&subagent_worktrees_dir());
    path.starts_with(data_dir) && !path.starts_with(subagent_worktrees)
}

fn subagent_worktrees_dir() -> PathBuf {
    crate::services::paths::data_dir().join("subagent-worktrees")
}

fn resolve_path(path: &str, working_dir: &Path) -> PathBuf {
    let raw = Path::new(path);
    let full = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        working_dir.join(raw)
    };
    canonical_or_normalized(&full)
}

fn canonical_or_normalized(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    if let (Some(parent), Some(file_name)) = (path.parent(), path.file_name()) {
        if let Ok(parent) = parent.canonicalize() {
            return parent.join(file_name);
        }
    }
    normalize_path(path)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
#[path = "permission_policy_tests.rs"]
mod tests;
