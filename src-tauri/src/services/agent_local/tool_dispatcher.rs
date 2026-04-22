use crate::services::agent_local::{
    tool_bash, tool_files, tool_glob, tool_grep, tool_web_fetch, tool_web_search,
};
use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;

pub use crate::services::agent_local::tool_definitions::{
    get_tool_definitions, get_chat_tool_definitions,
};

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
    let is_web = |n: &str| n == "web_search" || n == "web_fetch";
    let mut results: Vec<Option<ToolResult>> = vec![None; tool_calls.len()];

    let parallel_futs: Vec<_> = tool_calls
        .iter()
        .enumerate()
        .filter(|(_, (n, _))| !is_web(n))
        .map(|(i, (n, a))| {
            let fut = dispatch(n, a, working_dir);
            async move { (i, fut.await) }
        })
        .collect();

    for (i, result) in futures_util::future::join_all(parallel_futs).await {
        results[i] = Some(result);
    }

    for (i, (name, args)) in tool_calls.iter().enumerate() {
        if is_web(name) {
            results[i] = Some(dispatch(name, args, working_dir).await);
        }
    }

    results
        .into_iter()
        .map(|r| r.unwrap_or(ToolResult { content: String::new(), is_error: true }))
        .collect()
}
