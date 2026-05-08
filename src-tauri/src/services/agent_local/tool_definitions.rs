use serde_json::Value;

pub fn get_tool_definitions() -> Vec<Value> {
    let mut defs = vec![
        tool_def(
            "bash",
            "Execute a shell command. Commands run in the working directory. \
             Use for system commands, git, package managers, compilers, process management. \
             Default timeout 120s. For long-running commands, set timeout up to 600s.",
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
            "Read a file. Relative paths resolve from the working directory. \
             Output is formatted with line numbers. \
             Use offset and limit for large files. Default limit is 2000 lines.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path (relative to working directory, or absolute)"},
                    "offset": {"type": "integer", "description": "Starting line (0-based, default: 0)"},
                    "limit": {"type": "integer", "description": "Max lines to return (default: 2000)"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "write_file",
            "Create or overwrite a file. Relative paths resolve from the working directory.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path (relative to working directory, or absolute)"},
                    "content": {"type": "string", "description": "Content to write"}
                },
                "required": ["path", "content"]
            }),
        ),
        tool_def(
            "edit_file",
            "Modify a file by replacing an exact string match. Relative paths resolve from the working directory. \
             Prefer this over write_file for modifications.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path (relative to working directory, or absolute)"},
                    "old_string": {"type": "string", "description": "Exact text to find (must be unique in file)"},
                    "new_string": {"type": "string", "description": "Replacement text"}
                },
                "required": ["path", "old_string", "new_string"]
            }),
        ),
        tool_def(
            "list_dir",
            "List the contents of a directory. Use '.' to list the working directory.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Directory path (use '.' for working directory)"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "grep",
            "Search file contents with regex. Max 250 results, respects .gitignore. \
             Searches the working directory by default.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Regex pattern to search for"},
                    "path": {"type": "string", "description": "Directory to search (default: working directory)"},
                    "glob": {"type": "string", "description": "File filter glob (e.g. '*.rs', '*.ts')"}
                },
                "required": ["pattern"]
            }),
        ),
        tool_def(
            "glob",
            "Find files by name patterns. Max 100 results, respects .gitignore. \
             Searches the working directory by default.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Glob pattern (e.g. '**/*.ts', 'src/**/*.rs')"},
                    "path": {"type": "string", "description": "Root directory (default: working directory)"}
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
            "Load a skill by name for specialized workflows.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_name": {"type": "string", "description": "Exact skill name from the available skills list"}
                },
                "required": ["skill_name"]
            }),
        ),
        tool_def(
            "create_branch",
            "Create a new git branch from HEAD and switch to it. \
             Operates on the git repo in the working directory.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "branch_name": {"type": "string", "description": "Name for the new branch (e.g. feat/my-feature)"}
                },
                "required": ["branch_name"]
            }),
        ),
        tool_def(
            "checkout_branch",
            "Switch to an existing git branch. Fails if uncommitted changes. \
             Operates on the git repo in the working directory.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "branch_name": {"type": "string", "description": "Name of the branch to switch to"}
                },
                "required": ["branch_name"]
            }),
        ),
    ];
    defs.push(super::tool_definitions_subagent::delegate_task_definition());
    defs.extend(super::tool_definitions_office::office_tool_definitions());
    defs.extend(super::tool_definitions_mcp::mcp_tool_definitions());
    defs
}

pub fn get_chat_tool_definitions() -> Vec<Value> {
    let mut defs = vec![
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
    ];
    defs.extend(super::tool_definitions_mcp::mcp_tool_definitions());
    defs
}

pub(super) fn tool_def(name: &str, description: &str, parameters: Value) -> Value {
    serde_json::json!({
        "type": "function",
        "function": { "name": name, "description": description, "parameters": parameters }
    })
}
