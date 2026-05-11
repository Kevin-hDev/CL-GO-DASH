use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::{AgentMessage, AgentSession};

pub fn build_chat_messages(session: &AgentSession) -> Vec<ChatMessage> {
    session
        .messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
            images: None,
            tool_calls: None,
            tool_name: m.tool_name.clone(),
            tool_call_id: None,
            reasoning_content: None,
        })
        .collect()
}

pub fn new_user_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: content.to_string(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    }
}

pub fn chat_to_agent_message(m: &ChatMessage) -> Option<AgentMessage> {
    if m.role == "system" || m.role == "tool" {
        return None;
    }
    Some(AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: m.role.clone(),
        content: m.content.clone(),
        thinking: None,
        tool_calls: None,
        tool_name: m.tool_name.clone(),
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: 0,
        skill_names: None,
    })
}

pub fn new_user_agent_message(content: &str) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: content.to_string(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: 0,
        skill_names: None,
    }
}
