use serde_json::Value;

pub fn todo_write_definition() -> Value {
    super::tool_definitions::tool_def(
        "todo_write",
        "Create or update the current task checklist. Use this for multi-step coding tasks. \
         Send the full list each time, with at most one task marked in_progress.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "maxItems": 50,
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": {"type": "string", "description": "Short task name"},
                            "active_form": {"type": "string", "description": "Short present-tense label for an in-progress task"},
                            "status": {"type": "string", "enum": ["pending", "in_progress", "completed"]}
                        },
                        "required": ["content", "status"]
                    }
                }
            },
            "required": ["todos"]
        }),
    )
}
