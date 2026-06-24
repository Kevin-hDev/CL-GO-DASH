use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

#[test]
fn convert_extracts_system_as_instructions() {
    let msgs = vec![
        ChatMessage {
            role: "system".into(),
            content: "Tu es un assistant.".into(),
            ..Default::default()
        },
        ChatMessage {
            role: "user".into(),
            content: "Bonjour".into(),
            ..Default::default()
        },
    ];
    let (instructions, input) = convert_messages(&msgs);
    assert_eq!(instructions, "Tu es un assistant.");
    assert_eq!(input.len(), 1);
    assert_eq!(input[0]["role"], "user");
}

#[test]
fn convert_user_images_to_responses_parts() {
    let msgs = vec![ChatMessage {
        role: "user".into(),
        content: "Decris cette image".into(),
        images: Some(vec!["iVBORw0KGgo=".into()]),
        ..Default::default()
    }];
    let (_, input) = convert_messages(&msgs);
    assert_eq!(input[0]["role"], "user");
    assert_eq!(input[0]["content"][0]["type"], "input_text");
    assert_eq!(input[0]["content"][0]["text"], "Decris cette image");
    assert_eq!(input[0]["content"][1]["type"], "input_image");
    assert_eq!(
        input[0]["content"][1]["image_url"],
        "data:image/png;base64,iVBORw0KGgo="
    );
}

#[test]
fn convert_splits_tool_calls_into_separate_items() {
    let msgs = vec![
        ChatMessage {
            role: "assistant".into(),
            content: "Je vais lire le fichier.".into(),
            tool_calls: Some(vec![ToolCallOllama {
                id: Some("call_1".into()),
                extra_content: None,
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
