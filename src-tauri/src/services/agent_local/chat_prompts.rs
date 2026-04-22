use crate::services::agent_local::model_size::{self, PromptTier};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::{
    prompt_chat_compact, prompt_chat_detailed, prompt_compact, prompt_detailed,
};

fn build_system_message(content: String) -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content,
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
    }
}

fn prepend_tool_system_prompt(
    messages: &mut Vec<ChatMessage>,
    working_dir: &std::path::Path,
    model: &str,
) {
    if messages.first().is_some_and(|m| m.role == "system") {
        return;
    }
    let tier = model_size::detect_tier(model);
    let prompt = match tier {
        PromptTier::Compact => prompt_compact::build(working_dir),
        PromptTier::Detailed => prompt_detailed::build(working_dir),
    };
    messages.insert(0, build_system_message(prompt));
}

fn prepend_working_dir_context(
    messages: &mut Vec<ChatMessage>,
    working_dir: &std::path::Path,
) {
    let dir_info = format!(
        "You are working in the directory: {}. You can access any path on the system using absolute paths.",
        working_dir.display()
    );
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content = format!("{}\n\n{}", first.content, dir_info);
    } else {
        messages.insert(0, build_system_message(dir_info));
    }
}

pub fn prepend_agent_md_context(messages: &mut Vec<ChatMessage>, agent_md: Option<String>) {
    let content = match agent_md {
        Some(c) if !c.is_empty() => c,
        _ => return,
    };
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content = format!("{content}\n\n{}", first.content);
    } else {
        messages.insert(0, build_system_message(content));
    }
}

pub fn prepare_messages(
    messages: &mut Vec<ChatMessage>,
    working_dir: &std::path::Path,
    has_tools: bool,
    agent_md: Option<String>,
    skills: &[(String, String)],
    model: &str,
    mode: &str,
) {
    if mode == "chat" {
        prepend_chat_system_prompt(messages, working_dir, model);
    } else if has_tools {
        prepend_tool_system_prompt(messages, working_dir, model);
        if !skills.is_empty() {
            prepend_skills_listing(messages, skills);
        }
        prepend_agent_md_context(messages, agent_md);
    } else {
        prepend_working_dir_context(messages, working_dir);
    }
}

fn prepend_chat_system_prompt(
    messages: &mut Vec<ChatMessage>,
    working_dir: &std::path::Path,
    model: &str,
) {
    if messages.first().is_some_and(|m| m.role == "system") {
        return;
    }
    let tier = model_size::detect_tier(model);
    let prompt = match tier {
        PromptTier::Compact => prompt_chat_compact::build(working_dir),
        PromptTier::Detailed => prompt_chat_detailed::build(working_dir),
    };
    messages.insert(0, build_system_message(prompt));
}

fn prepend_skills_listing(messages: &mut Vec<ChatMessage>, skills: &[(String, String)]) {
    let listing = skills
        .iter()
        .map(|(name, desc)| format!("- {name}: {desc}"))
        .collect::<Vec<_>>()
        .join("\n");

    let section = format!(
        "\n\n## Available skills\n\
         The following skills are available. Use the `load_skill` tool to load one when relevant.\n\
         {listing}"
    );

    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content.push_str(&section);
    }
}
