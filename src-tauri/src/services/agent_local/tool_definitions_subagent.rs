use serde_json::Value;

pub fn delegate_task_definition() -> Value {
    super::tool_definitions::tool_def(
        "delegate_task",
        "Spawn an autonomous subagent to execute a subtask in the background. \
         Types: 'explorer' (read-only: file search, code reading, web research) \
         or 'coder' (file creation/modification in an isolated git worktree). \
         You can spawn multiple subagents in parallel for independent subtasks. \
         IMPORTANT: Once you delegate a task, do NOT perform the same work yourself. \
         Wait for the subagent reports, then synthesize their results for the user. \
         The subagent results are NOT visible to the user — you MUST relay them.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Structured instruction for the subagent. Use XML tags: \
                     <context> (background info the subagent needs), \
                     <task> (what to do — be specific), \
                     <constraints> (boundaries, what NOT to do), \
                     <output_format> (expected response structure). \
                     A well-structured prompt produces significantly better results."
                },
                "subagent_type": {
                    "type": "string",
                    "enum": ["explorer", "coder"],
                    "description": "explorer = read-only research, coder = file modifications in isolated worktree"
                },
                "name": {
                    "type": "string",
                    "description": "Short name to identify this subagent in the UI (optional)"
                }
            },
            "required": ["prompt", "subagent_type"]
        }),
    )
}

pub fn get_explorer_tool_definitions() -> Vec<Value> {
    use super::tool_definitions::tool_def;
    vec![
        tool_def(
            "read_file",
            "Read any file. Relative paths resolve from the working directory. Output is formatted with line numbers.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path"},
                    "offset": {"type": "integer", "description": "Starting line (0-based)"},
                    "limit": {"type": "integer", "description": "Max lines (default: 2000)"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "list_dir",
            "List the contents of a directory. Use '.' to list the working directory.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Directory path"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "grep",
            "Search file contents with regex patterns. Searches the working directory by default.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Regex pattern"},
                    "path": {"type": "string", "description": "Directory to search in"},
                    "glob": {"type": "string", "description": "File filter glob"}
                },
                "required": ["pattern"]
            }),
        ),
        tool_def(
            "glob",
            "Find files by name patterns. Searches the working directory by default.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Glob pattern"},
                    "path": {"type": "string", "description": "Root directory"}
                },
                "required": ["pattern"]
            }),
        ),
        tool_def(
            "web_search",
            "Search the web for current information.",
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
    ]
}
