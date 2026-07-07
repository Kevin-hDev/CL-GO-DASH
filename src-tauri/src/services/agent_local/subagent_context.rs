use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::agent_local::types_session::{AgentMessage, ToolCallRequest};

pub async fn build_messages(
    child_session_id: &str,
    system_prompt: String,
    fallback_prompt: &str,
) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
        ..Default::default()
    }];

    if let Ok(child) = super::session_store::get(child_session_id).await {
        messages.extend(child.messages.into_iter().filter_map(saved_to_chat));
    }

    if messages.len() == 1 {
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: fallback_prompt.to_string(),
            ..Default::default()
        });
    }

    messages
}

fn saved_to_chat(message: AgentMessage) -> Option<ChatMessage> {
    if !matches!(message.role.as_str(), "user" | "assistant" | "tool") {
        return None;
    }
    let tool_calls = message.tool_calls.map(convert_tool_calls);
    if message.content.trim().is_empty() && tool_calls.as_ref().is_none_or(Vec::is_empty) {
        return None;
    }
    Some(ChatMessage {
        role: message.role,
        content: message.content,
        images: None,
        tool_calls,
        tool_name: message.tool_name,
        tool_call_id: None,
        reasoning_content: None,
    })
}

fn convert_tool_calls(calls: Vec<ToolCallRequest>) -> Vec<ToolCallOllama> {
    calls
        .into_iter()
        .map(|call| ToolCallOllama {
            id: None,
            extra_content: call.extra_content,
            function: ToolCallFunction {
                name: call.function.name,
                arguments: call.function.arguments,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(role: &str, content: &str) -> AgentMessage {
        AgentMessage {
            id: "m1".into(),
            role: role.into(),
            content: content.into(),
            thinking: None,
            tool_calls: None,
            tool_name: None,
            tool_activities: None,
            segments: None,
            files: vec![],
            timestamp: chrono::Utc::now(),
            tokens: 0,
            work_duration_ms: None,
            skill_names: None,
        }
    }

    #[test]
    fn saved_to_chat_keeps_supported_roles() {
        assert!(saved_to_chat(saved("user", "Suite")).is_some());
        assert!(saved_to_chat(saved("assistant", "Ok")).is_some());
        assert!(saved_to_chat(saved("system", "Ignore")).is_none());
    }
}
