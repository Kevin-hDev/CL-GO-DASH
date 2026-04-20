use crate::services::agent_local::{
    tool_bash, tool_files, tool_glob, tool_grep, tool_web_fetch, tool_web_search,
};
use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;

pub async fn dispatch(tool_name: &str, args: &Value, working_dir: &Path) -> ToolResult {
    match tool_name {
        "bash" => {
            let cmd = args["command"].as_str().unwrap_or("");
            let timeout = args["timeout"].as_u64();
            match tool_bash::execute_shell(cmd, working_dir, timeout).await {
                Ok(out) => ToolResult {
                    content: format!("{}\n{}", out.stdout, out.stderr).trim().to_string(),
                    is_error: out.exit_code != 0,
                },
                Err(e) => ToolResult { content: e, is_error: true },
            }
        }
        "read_file" => {
            let path = args["path"].as_str().unwrap_or("");
            tool_files::read_file(path, working_dir).await
        }
        "write_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let content = args["content"].as_str().unwrap_or("");
            tool_files::write_file(path, content, working_dir).await
        }
        "edit_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let old = args["old_string"].as_str().unwrap_or("");
            let new = args["new_string"].as_str().unwrap_or("");
            tool_files::edit_file(path, old, new, working_dir).await
        }
        "list_dir" => {
            let path = args["path"].as_str().unwrap_or(".");
            tool_files::list_dir(path, working_dir).await
        }
        "grep" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let path = args["path"].as_str();
            let glob_filter = args["glob"].as_str();
            tool_grep::grep(pattern, path, glob_filter, working_dir).await
        }
        "glob" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let path = args["path"].as_str();
            tool_glob::glob_files(pattern, path, working_dir).await
        }
        "web_search" => {
            let query = args["query"].as_str().unwrap_or("");
            match tool_web_search::web_search(query).await {
                Ok(results) => {
                    let text = results
                        .iter()
                        .map(|r| format!("**{}**\n{}\n{}", r.title, r.url, r.snippet))
                        .collect::<Vec<_>>()
                        .join("\n\n");
                    ToolResult { content: text, is_error: false }
                }
                Err(e) => ToolResult { content: e, is_error: true },
            }
        }
        "web_fetch" => {
            let url = args["url"].as_str().unwrap_or("");
            match tool_web_fetch::fetch_url(url).await {
                Ok(content) => ToolResult { content, is_error: false },
                Err(e) => ToolResult { content: e, is_error: true },
            }
        }
        "load_skill" => {
            let name = args["skill_name"].as_str().unwrap_or("");
            match tool_skill_loader::load_skill(name).await {
                Ok(content) => ToolResult {
                    content: format!("Skill '{name}' loaded. Follow its instructions:\n\n{content}"),
                    is_error: false,
                },
                Err(e) => ToolResult { content: e, is_error: true },
            }
        }
        _ => ToolResult {
            content: format!("Outil inconnu: {tool_name}"),
            is_error: true,
        },
    }
}

pub async fn dispatch_multiple(
    tool_calls: &[(String, Value)],
    working_dir: &Path,
) -> Vec<ToolResult> {
    let futures: Vec<_> = tool_calls
        .iter()
        .map(|(name, args)| dispatch(name, args, working_dir))
        .collect();
    futures_util::future::join_all(futures).await
}

pub fn get_tool_definitions() -> Vec<Value> {
    vec![
        tool_def(
            "bash",
            "Execute any shell command on the user's system. Use for system commands, git, \
             package managers, compilers, process management, and any task requiring shell execution. \
             Default timeout is 120s (2 min). For long-running commands (du on large dirs, builds, \
             installs), set a higher timeout up to 600s (10 min).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command to execute"},
                    "timeout": {"type": "integer", "description": "Timeout in seconds (default: 120, max: 600)"}
                },
                "required": ["command"]
            }),
        ),
        tool_def(
            "read_file",
            "Read any file on the system. Accepts absolute or relative paths. \
             You can access any file the user can access.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path (absolute or relative to working dir)"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "write_file",
            "Create a new file or overwrite an existing file.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path to write to"},
                    "content": {"type": "string", "description": "Content to write"}
                },
                "required": ["path", "content"]
            }),
        ),
        tool_def(
            "edit_file",
            "Modify an existing file by replacing an exact string match. \
             Prefer this over write_file for modifications.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path to edit"},
                    "old_string": {"type": "string", "description": "Exact text to find (must be unique in file)"},
                    "new_string": {"type": "string", "description": "Replacement text"}
                },
                "required": ["path", "old_string", "new_string"]
            }),
        ),
        tool_def(
            "list_dir",
            "List the contents of a directory.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Directory path to list"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "grep",
            "Search file contents with regex patterns. Max 250 results, respects .gitignore.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Regex pattern to search for"},
                    "path": {"type": "string", "description": "Directory to search in (default: working dir)"},
                    "glob": {"type": "string", "description": "File filter glob (e.g. '*.rs', '*.ts')"}
                },
                "required": ["pattern"]
            }),
        ),
        tool_def(
            "glob",
            "Find files by name patterns. Max 100 results, respects .gitignore.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Glob pattern (e.g. '**/*.ts', 'src/**/*.rs')"},
                    "path": {"type": "string", "description": "Root directory (default: working dir)"}
                },
                "required": ["pattern"]
            }),
        ),
        tool_def(
            "web_search",
            "Search the web for current information, documentation, or solutions.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"}
                },
                "required": ["query"]
            }),
        ),
        tool_def(
            "web_fetch",
            "Fetch and extract content from a URL.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to fetch"}
                },
                "required": ["url"]
            }),
        ),
        tool_def(
            "load_skill",
            "Load a skill by name for specialized workflows. \
             Use when the user mentions a skill or the task matches an available skill.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_name": {"type": "string", "description": "Exact skill name from the available skills list"}
                },
                "required": ["skill_name"]
            }),
        ),
    ]
}

fn tool_def(name: &str, description: &str, parameters: Value) -> Value {
    serde_json::json!({
        "type": "function",
        "function": { "name": name, "description": description, "parameters": parameters }
    })
}
