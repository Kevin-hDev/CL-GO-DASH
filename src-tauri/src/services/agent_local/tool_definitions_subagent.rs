use serde_json::Value;

pub fn delegate_task_definition() -> Value {
    super::tool_definitions::tool_def(
        "delegate_task",
        "Lance un sous-agent autonome pour exécuter une sous-tâche en arrière-plan. \
         Le sous-agent travaille de façon indépendante pendant que tu continues. \
         Type 'explorer' : recherche de fichiers, lecture de code, recherche web (read-only). \
         Type 'coder' : création et modification de fichiers dans un worktree git isolé. \
         Tu peux lancer plusieurs sous-agents en parallèle. \
         Le résultat du sous-agent n'est PAS visible par l'utilisateur — tu dois le relayer.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Instruction détaillée pour le sous-agent"
                },
                "subagent_type": {
                    "type": "string",
                    "enum": ["explorer", "coder"],
                    "description": "explorer = lecture seule, coder = modification fichiers dans worktree isolé"
                },
                "name": {
                    "type": "string",
                    "description": "Nom court pour identifier le sous-agent dans l'UI (optionnel)"
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
            "Read any file on the system. Output is formatted with line numbers.",
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
            "List the contents of a directory.",
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
            "Search file contents with regex patterns.",
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
            "Find files by name patterns.",
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
