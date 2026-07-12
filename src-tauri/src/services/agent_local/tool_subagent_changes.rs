use super::types_tools::ToolResult;
use serde_json::{json, Value};
use std::path::Path;

pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    parent_id: &str,
) -> Option<ToolResult> {
    if !matches!(
        tool_name,
        "inspect_subagent_changes" | "apply_subagent_changes" | "discard_subagent_changes"
    ) {
        return None;
    }
    let child_id = match id_arg(args, "subagent_id") {
        Ok(value) => value,
        Err(result) => return Some(result),
    };
    let change_id = match id_arg(args, "change_id") {
        Ok(value) => value,
        Err(result) => return Some(result),
    };
    let result = match tool_name {
        "inspect_subagent_changes" => {
            match super::subagent_git_actions::inspect(
                working_dir,
                parent_id,
                child_id,
                change_id,
            )
            .await
            {
                Ok((change, patch, truncated)) => ToolResult {
                    content: json!({
                        "change": change,
                        "patch": patch,
                        "patch_truncated": truncated
                    })
                    .to_string(),
                    is_error: false,
                    truncated,
                    affected_paths: Vec::new(),
                },
                Err(_) => unavailable(),
            }
        }
        "apply_subagent_changes" => action_result(
            super::subagent_git_actions::apply(working_dir, parent_id, child_id, change_id).await,
            "Application du changement sous-agent impossible.",
        ),
        "discard_subagent_changes" => action_result(
            super::subagent_git_actions::discard(working_dir, parent_id, child_id, change_id).await,
            "Abandon du changement sous-agent impossible.",
        ),
        _ => unreachable!(),
    };
    Some(result)
}

fn action_result(
    result: Result<super::types_subagent_change::SubagentChangeMeta, String>,
    error: &str,
) -> ToolResult {
    match result {
        Ok(change) => {
            let paths = change
                .changed_paths
                .iter()
                .map(|changed| changed.path.clone())
                .collect();
            ToolResult::ok(json!({ "change": change }).to_string()).with_affected_paths(paths)
        }
        Err(_) => ToolResult::err(error),
    }
}

fn id_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, ToolResult> {
    let value = args[key].as_str().ok_or_else(unavailable)?;
    super::types_subagent_change::validate_uuid(value).map_err(|_| unavailable())?;
    Ok(value)
}

fn unavailable() -> ToolResult {
    ToolResult::err("Changement sous-agent indisponible.")
}
