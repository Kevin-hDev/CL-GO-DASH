use serde_json::Value;

pub fn delegate_task_definition() -> Value {
    super::tool_definitions::tool_def(
        "delegate_task",
        "Spawn an autonomous subagent to handle a subtask in isolation. Results come back to you; they are NOT shown to the user, so you must relay a summary. \
         Types: 'explorer' (read-only: read_file, list_dir, grep, glob, web_search, web_fetch) for research, file investigation, web lookups; \
         or 'coder' (file creation/modification in an isolated git worktree) for parallel implementation work. \
         When to use: independent subtasks that can run in parallel; open-ended research that would take several rounds of grep/glob/read; background work you don't need immediately. \
         When NOT to use: reading a specific known file — use read_file directly; searching for a single class/function — use grep or glob directly; a 1-2 step task — do it yourself. \
         IMPORTANT: once you delegate a task, do NOT do the same work yourself. Wait for the subagent report, then synthesize. \
         Write a structured prompt using XML tags: <context>, <task>, <constraints>, <output_format>. Terse prompts produce shallow results. \
         You can spawn multiple subagents in parallel for independent subtasks.",
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
