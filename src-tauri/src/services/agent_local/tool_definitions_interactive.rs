use serde_json::Value;

pub fn ask_user_choice_definition() -> Value {
    super::tool_definitions::tool_def(
        "ask_user_choice",
        "Ask the user to choose between concrete options when their decision changes the next step. \
         Use sparingly unless an active workflow requires it. \
         In Plan Mode, this is the mandatory way to ask user questions before publishing a plan or before final approval. \
         Do not ask those Plan Mode questions in normal assistant text. \
         Always include a recommended option when useful.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "questions": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 4,
                    "items": {
                        "type": "object",
                        "properties": {
                            "header": {"type": "string", "description": "Very short label, max 12 characters"},
                            "question": {"type": "string", "description": "Question shown to the user"},
                            "multi_select": {"type": "boolean", "description": "Allow multiple options"},
                            "options": {
                                "type": "array",
                                "minItems": 2,
                                "maxItems": 4,
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "label": {"type": "string"},
                                        "description": {"type": "string"},
                                        "recommended": {"type": "boolean"},
                                        "preview": {"type": "string"}
                                    },
                                    "required": ["label", "description"]
                                }
                            }
                        },
                        "required": ["header", "question", "options"]
                    }
                }
            },
            "required": ["questions"]
        }),
    )
}
