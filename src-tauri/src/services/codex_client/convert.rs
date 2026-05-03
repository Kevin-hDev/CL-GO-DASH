use crate::services::agent_local::types_ollama::ChatMessage;

pub fn convert_messages(messages: &[ChatMessage]) -> (String, Vec<serde_json::Value>) {
    let mut instructions = String::new();
    let mut input = Vec::new();

    for msg in messages {
        if msg.role == "system" {
            if !instructions.is_empty() {
                instructions.push_str("\n\n");
            }
            instructions.push_str(&msg.content);
            continue;
        }

        if msg.role == "assistant" {
            if !msg.content.is_empty() {
                input.push(serde_json::json!({"role": "assistant", "content": msg.content}));
            }
            if let Some(ref calls) = msg.tool_calls {
                for tc in calls {
                    let args = match &tc.function.arguments {
                        serde_json::Value::String(s) => s.clone(),
                        other => serde_json::to_string(other).unwrap_or_default(),
                    };
                    input.push(serde_json::json!({
                        "type": "function_call",
                        "call_id": tc.id.as_deref().unwrap_or("call_0"),
                        "name": tc.function.name,
                        "arguments": args,
                    }));
                }
            }
            continue;
        }

        if msg.role == "tool" {
            if let Some(ref id) = msg.tool_call_id {
                input.push(serde_json::json!({
                    "type": "function_call_output",
                    "call_id": id,
                    "output": msg.content,
                }));
                continue;
            }
        }

        input.push(serde_json::json!({"role": msg.role, "content": msg.content}));
    }
    (instructions, input)
}

fn fix_array_schemas(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            if map.get("type").and_then(|t| t.as_str()) == Some("array")
                && !map.contains_key("items")
            {
                map.insert("items".to_string(), serde_json::json!({"type": "string"}));
            }
            for val in map.values_mut() {
                fix_array_schemas(val);
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr {
                fix_array_schemas(val);
            }
        }
        _ => {}
    }
}

pub fn convert_tools_to_responses_api(tools: &[serde_json::Value]) -> Vec<serde_json::Value> {
    tools
        .iter()
        .filter_map(|t| {
            let func = t.get("function")?;
            let mut params = func.get("parameters").cloned().unwrap_or(serde_json::Value::Null);
            fix_array_schemas(&mut params);
            Some(serde_json::json!({
                "type": "function",
                "name": func.get("name")?,
                "description": func.get("description").unwrap_or(&serde_json::Value::Null),
                "parameters": params,
            }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

    #[test]
    fn convert_extracts_system_as_instructions() {
        let msgs = vec![
            ChatMessage { role: "system".into(), content: "Tu es un assistant.".into(), ..Default::default() },
            ChatMessage { role: "user".into(), content: "Bonjour".into(), ..Default::default() },
        ];
        let (instructions, input) = convert_messages(&msgs);
        assert_eq!(instructions, "Tu es un assistant.");
        assert_eq!(input.len(), 1);
        assert_eq!(input[0]["role"], "user");
    }

    #[test]
    fn convert_handles_no_system() {
        let msgs = vec![ChatMessage { role: "user".into(), content: "Hello".into(), ..Default::default() }];
        let (instructions, input) = convert_messages(&msgs);
        assert!(instructions.is_empty());
        assert_eq!(input.len(), 1);
    }

    #[test]
    fn convert_splits_tool_calls_into_separate_items() {
        let msgs = vec![
            ChatMessage {
                role: "assistant".into(),
                content: "Je vais lire le fichier.".into(),
                tool_calls: Some(vec![ToolCallOllama {
                    id: Some("call_1".into()),
                    function: ToolCallFunction {
                        name: "read_file".into(),
                        arguments: serde_json::json!({"path": "/tmp/test.txt"}),
                    },
                }]),
                ..Default::default()
            },
            ChatMessage {
                role: "tool".into(),
                content: "contenu du fichier".into(),
                tool_call_id: Some("call_1".into()),
                ..Default::default()
            },
        ];
        let (_, input) = convert_messages(&msgs);
        assert_eq!(input.len(), 3);
        assert_eq!(input[0]["role"], "assistant");
        assert_eq!(input[1]["type"], "function_call");
        assert_eq!(input[1]["name"], "read_file");
        assert_eq!(input[1]["call_id"], "call_1");
        assert_eq!(input[2]["type"], "function_call_output");
        assert_eq!(input[2]["call_id"], "call_1");
    }
}
