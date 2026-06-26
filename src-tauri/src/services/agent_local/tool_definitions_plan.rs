use serde_json::Value;

pub fn planmode_definition() -> Value {
    super::tool_definitions::tool_def(
        "planmode",
        "Publish or update the current implementation plan while Plan mode is active. \
         Use only after read-only exploration and after every important question has been answered. \
         After this tool succeeds, you must call ask_user_choice for final approval before any implementation. \
         Do not ask approval in normal assistant text.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Short plan title"},
                "content": {"type": "string", "description": "Markdown plan content"}
            },
            "required": ["title", "content"]
        }),
    )
}

pub fn exitplanmode_definition() -> Value {
    super::tool_definitions::tool_def(
        "exitplanmode",
        "Exit Plan mode only after the user answered the final ask_user_choice plan approval. \
         When status is approved and this tool succeeds, immediately start implementation.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["approved", "rejected", "cancelled"],
                    "description": "Final plan decision"
                }
            },
            "required": ["status"]
        }),
    )
}
