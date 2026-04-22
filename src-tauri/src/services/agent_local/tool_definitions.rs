use serde_json::Value;

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

pub fn get_chat_tool_definitions() -> Vec<Value> {
    vec![
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
    ]
}

fn tool_def(name: &str, description: &str, parameters: Value) -> Value {
    serde_json::json!({
        "type": "function",
        "function": { "name": name, "description": description, "parameters": parameters }
    })
}
