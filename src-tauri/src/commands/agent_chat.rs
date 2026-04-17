use crate::services::agent_local::agent_loop;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::{session_store, tab_store};
use crate::services::agent_local::types_ollama::{
    ChatMessage, StreamEvent,
};
use crate::services::agent_local::types_session::{
    AgentMessage, AgentSession, AgentSessionMeta, TabState,
};
use crate::services::llm;
use crate::ActiveStreams;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn chat_stream(
    session_id: String,
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<serde_json::Value>,
    think: bool,
    provider: Option<String>,
    working_dir: Option<String>,
    on_event: Channel<StreamEvent>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    let cancel = CancellationToken::new();
    streams.0.lock().await.insert(session_id.clone(), cancel.clone());

    let provider = provider.unwrap_or_else(|| "ollama".to_string());

    let resolve_dir = |wd: &Option<String>| -> std::path::PathBuf {
        wd.as_ref()
            .map(std::path::PathBuf::from)
            .filter(|p| p.is_dir())
            .unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| dirs::home_dir().unwrap())
            })
    };

    let result = if provider == "ollama" {
        let final_tools = if tools.is_empty() {
            tool_dispatcher::get_tool_definitions()
        } else {
            tools
        };

        let working_dir = resolve_dir(&working_dir);
        let mut msgs = messages;
        prepend_working_dir_context(&mut msgs, &working_dir);

        agent_loop::run_agent_loop(
            &on_event,
            &mut msgs,
            &model,
            final_tools,
            think,
            working_dir,
            session_id.clone(),
            cancel,
        )
        .await
        .map(|_| ())
    } else {
        // Chat LLM API : tools uniquement si le modèle les supporte.
        use crate::services::llm::tool_capable;
        let model_supports = tool_capable::supports_tools(&provider, &model);
        let final_tools = if model_supports {
            if tools.is_empty() { tool_dispatcher::get_tool_definitions() } else { tools }
        } else {
            vec![]
        };
        let openai_tools = llm::agent_loop::convert_tools_to_openai(&final_tools);
        let working_dir = resolve_dir(&working_dir);
        let mut msgs = messages;
        if !openai_tools.is_empty() {
            prepend_tool_system_prompt(&mut msgs, &working_dir);
        } else {
            prepend_working_dir_context(&mut msgs, &working_dir);
        }
        llm::agent_loop::run_agent_loop(
            &on_event,
            &provider,
            &model,
            &mut msgs,
            &openai_tools,
            working_dir,
            session_id.clone(),
            cancel,
        )
        .await
        .map(|_| ())
    };

    streams.0.lock().await.remove(&session_id);
    result
}

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
- Never guess file content — read it first
- If you lack information to call a tool, ask the user
- Keep going until the task is fully resolved
- When in doubt: respond directly, no tool needed";

fn prepend_tool_system_prompt(messages: &mut Vec<ChatMessage>, working_dir: &std::path::Path) {
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

fn prepend_working_dir_context(messages: &mut Vec<ChatMessage>, working_dir: &std::path::Path) {
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

#[tauri::command]
pub async fn cancel_agent_request(
    session_id: String,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    if let Some(token) = streams.0.lock().await.get(&session_id) {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn list_agent_sessions() -> Result<Vec<AgentSessionMeta>, String> {
    session_store::list().await
}

#[tauri::command]
pub async fn get_agent_session(id: String) -> Result<AgentSession, String> {
    session_store::get(&id).await
}

#[tauri::command]
pub async fn save_agent_session(session: AgentSession) -> Result<(), String> {
    session_store::save(&session).await
}

#[tauri::command]
pub async fn add_messages_to_session(
    id: String,
    messages: Vec<crate::services::agent_local::types_session::AgentMessage>,
    tokens: u32,
) -> Result<(), String> {
    session_store::add_messages(&id, messages, tokens).await
}

#[tauri::command]
pub async fn create_agent_session(
    name: String,
    model: String,
    provider: Option<String>,
    project_id: Option<String>,
) -> Result<AgentSession, String> {
    let provider = provider.unwrap_or_else(|| "ollama".to_string());
    session_store::create_full(&name, &model, &provider, false, project_id).await
}

#[tauri::command]
pub async fn rename_agent_session(id: String, name: String) -> Result<(), String> {
    session_store::rename(&id, &name).await
}

#[tauri::command]
pub async fn delete_agent_session(id: String) -> Result<(), String> {
    session_store::delete(&id).await
}

#[tauri::command]
pub async fn export_agent_session_markdown(id: String) -> Result<String, String> {
    session_store::export_markdown(&id).await
}

#[tauri::command]
pub async fn truncate_session_at(
    session_id: String,
    message_id: String,
) -> Result<(), String> {
    session_store::truncate_at(&session_id, &message_id).await
}

#[tauri::command]
pub async fn truncate_and_replace_at(
    session_id: String,
    message_id: String,
    replacement: Option<AgentMessage>,
) -> Result<(), String> {
    session_store::truncate_and_replace(&session_id, &message_id, replacement).await
}

#[tauri::command]
pub async fn get_tab_state() -> Result<TabState, String> {
    tab_store::get_state().await
}

#[tauri::command]
pub async fn save_tab_state(state: TabState) -> Result<(), String> {
    tab_store::save_state(&state).await
}
