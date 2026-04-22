use crate::services::agent_local::agent_loop;
use crate::services::agent_local::agent_md;
use crate::services::agent_local::agent_settings;
use crate::services::agent_local::chat_prompts::prepare_messages;
use crate::services::personality_injection;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::llm;
use crate::ActiveStreams;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn chat_stream(
    app: tauri::AppHandle,
    session_id: String,
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<serde_json::Value>,
    think: bool,
    provider: Option<String>,
    working_dir: Option<String>,
    supports_tools: Option<bool>,
    supports_thinking: Option<bool>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    const MAX_ACTIVE_STREAMS: usize = 32;
    let cancel = CancellationToken::new();
    {
        let mut map = streams.0.lock().await;
        if map.len() >= MAX_ACTIVE_STREAMS {
            return Err("Trop de flux actifs simultanément".to_string());
        }
        map.insert(session_id.clone(), cancel.clone());
    }

    let provider = provider.unwrap_or_else(|| "ollama".to_string());
    let stream_session = session_id.clone();
    let task_app = app.clone();

    tauri::async_runtime::spawn(async move {
        let emitter = AgentEventEmitter::new(task_app.clone(), stream_session.clone());
        let result = run_stream_task(
            emitter.clone(),
            stream_session.clone(),
            model,
            messages,
            tools,
            think,
            provider,
            working_dir,
            supports_tools,
            supports_thinking,
            cancel,
        )
        .await;

        task_app
            .state::<ActiveStreams>()
            .0
            .lock()
            .await
            .remove(&stream_session);

        crate::services::agent_local::permission_gate::clear_session(&stream_session).await;
        crate::services::agent_local::session_store::remove_session_lock(&stream_session).await;

        if let Err(message) = result {
            let _ = emitter.send(StreamEvent::Error { message });
        }
    });

    Ok(())
}

async fn run_stream_task(
    on_event: AgentEventEmitter,
    session_id: String,
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<serde_json::Value>,
    think: bool,
    provider: String,
    working_dir: Option<String>,
    supports_tools_hint: Option<bool>,
    supports_thinking_hint: Option<bool>,
    cancel: CancellationToken,
) -> Result<(), String> {
    let resolve_dir = |wd: &Option<String>| -> std::path::PathBuf {
        wd.as_ref()
            .map(std::path::PathBuf::from)
            .filter(|p| p.is_dir())
            .unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| dirs::home_dir().unwrap())
            })
    };

    let mode = agent_settings::get_permission_mode().await;
    let is_chat = mode == "chat";

    if provider == "ollama" {
        let final_tools = if tools.is_empty() {
            if is_chat {
                tool_dispatcher::get_chat_tool_definitions()
            } else {
                tool_dispatcher::get_tool_definitions()
            }
        } else {
            tools
        };

        let working_dir = resolve_dir(&working_dir);
        let mut msgs = messages;
        let agent_md_content = if is_chat {
            None
        } else {
            let raw = agent_md::load_agent_md(Some(working_dir.as_path())).await;
            let personality = personality_injection::load_injected_contents();
            merge_personality(raw, personality)
        };
        let skills_tuples: Vec<(String, String)> = if is_chat {
            vec![]
        } else {
            tool_skill_loader::list_skills()
                .await
                .unwrap_or_default()
                .iter()
                .map(|s| (s.name.clone(), s.description.clone()))
                .collect()
        };
        prepare_messages(&mut msgs, &working_dir, true, agent_md_content, &skills_tuples, &model, &mode);

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
        use crate::services::llm::{model_registry, tool_capable};
        let registry_caps = model_registry::lookup(&provider, &model).await;
        let model_supports_tools = supports_tools_hint.unwrap_or_else(|| {
            registry_caps.as_ref().map(|c| c.supports_tools).unwrap_or(false)
                || tool_capable::supports_tools(&provider, &model)
        });
        let model_supports_thinking = supports_thinking_hint.unwrap_or_else(|| {
            registry_caps.as_ref().map(|c| c.supports_thinking).unwrap_or(false)
                || tool_capable::supports_thinking(&provider, &model)
        });

        let model_supports_vision = registry_caps.as_ref().map(|c| c.supports_vision).unwrap_or(false)
            || tool_capable::supports_vision(&provider, &model);

        let final_tools = if is_chat {
            tool_dispatcher::get_chat_tool_definitions()
        } else if model_supports_tools {
            if tools.is_empty() { tool_dispatcher::get_tool_definitions() } else { tools }
        } else {
            vec![]
        };
        let openai_tools = llm::agent_loop::convert_tools_to_openai(&final_tools);
        let working_dir = resolve_dir(&working_dir);
        let mut msgs = messages;
        if !model_supports_vision {
            llm::stream_convert::strip_images(&mut msgs);
        }
        let agent_md_content = if is_chat {
            None
        } else {
            let raw = agent_md::load_agent_md(Some(working_dir.as_path())).await;
            let personality = personality_injection::load_injected_contents();
            merge_personality(raw, personality)
        };
        let skills_tuples: Vec<(String, String)> = if is_chat || !model_supports_tools {
            vec![]
        } else {
            tool_skill_loader::list_skills()
                .await
                .unwrap_or_default()
                .iter()
                .map(|s| (s.name.clone(), s.description.clone()))
                .collect()
        };
        let has_tools = is_chat || model_supports_tools;
        prepare_messages(&mut msgs, &working_dir, has_tools, agent_md_content, &skills_tuples, &model, &mode);
        let think_active = think && model_supports_thinking;
        llm::agent_loop::run_agent_loop(
            &on_event,
            &provider,
            &model,
            &mut msgs,
            &openai_tools,
            think_active,
            working_dir,
            session_id.clone(),
            cancel,
        )
        .await
        .map(|_| ())
    }
}

fn merge_personality(agent_md: Option<String>, personality: Option<String>) -> Option<String> {
    match (agent_md, personality) {
        (Some(a), Some(p)) => Some(format!("{a}\n\n{p}")),
        (Some(a), None) => Some(a),
        (None, Some(p)) => Some(p),
        (None, None) => None,
    }
}

#[tauri::command]
pub async fn cancel_agent_request(
    session_id: String,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    if let Some(token) = streams.0.lock().await.remove(&session_id) {
        token.cancel();
    }
    Ok(())
}
