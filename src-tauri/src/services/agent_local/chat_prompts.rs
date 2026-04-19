use crate::services::agent_local::types_ollama::ChatMessage;

const TOOL_SYSTEM_PROMPT: &str = "\
You are a helpful assistant with access to tools.

## When to use tools
Use a tool ONLY when:
- The user asks to act on files (read, write, edit)
- The user asks to run a command
- The user asks for a web search or recent information
- You cannot answer accurately without external data

## When NOT to use tools
Respond directly WITHOUT any tool for:
- Normal conversation
- Creative tasks (stories, poems, essays)
- Questions you can answer from your own knowledge
- Explanations, summaries, translations

## Rules
- Think BEFORE each tool call: is it truly necessary?
- Never guess file content - read it first
- If you lack information to call a tool, ask the user
- Keep going until the task is fully resolved
- When in doubt: respond directly, no tool needed";

pub fn prepend_tool_system_prompt(messages: &mut Vec<ChatMessage>, working_dir: &std::path::Path) {
    let has_system = messages.first().is_some_and(|m| m.role == "system");
    if has_system {
        return;
    }
    let dir_info = format!(
        "\n\n## Working directory\nYou are working in: {}\nAll file paths are relative to this directory unless specified otherwise.",
        working_dir.display()
    );
    messages.insert(0, ChatMessage {
        role: "system".to_string(),
        content: format!("{TOOL_SYSTEM_PROMPT}{dir_info}"),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
    });
}

pub fn prepend_working_dir_context(
    messages: &mut Vec<ChatMessage>,
    working_dir: &std::path::Path,
) {
    let has_system = messages.first().is_some_and(|m| m.role == "system");
    let dir_info = format!(
        "You are working in the directory: {}. All file operations use this as the base directory.",
        working_dir.display()
    );
    if has_system {
        if let Some(first) = messages.first_mut() {
            first.content = format!("{}\n\n{}", first.content, dir_info);
        }
    } else {
        messages.insert(0, ChatMessage {
            role: "system".to_string(),
            content: dir_info,
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        });
    }
}

pub fn prepend_agent_md_context(messages: &mut Vec<ChatMessage>, agent_md: Option<String>) {
    let content = match agent_md {
        Some(c) if !c.is_empty() => c,
        _ => return,
    };

    let has_system = messages.first().is_some_and(|m| m.role == "system");
    if has_system {
        if let Some(first) = messages.first_mut() {
            first.content = format!("{}\n\n{}", content, first.content);
        }
    } else {
        messages.insert(0, ChatMessage {
            role: "system".to_string(),
            content,
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        });
    }
}

pub fn prepare_messages(
    messages: &mut Vec<ChatMessage>,
    working_dir: &std::path::Path,
    has_tools: bool,
    agent_md: Option<String>,
) {
    if has_tools {
        prepend_tool_system_prompt(messages, working_dir);
        prepend_agent_md_context(messages, agent_md);
    } else {
        prepend_working_dir_context(messages, working_dir);
    }
}
