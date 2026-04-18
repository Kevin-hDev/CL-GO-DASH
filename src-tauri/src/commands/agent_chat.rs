use crate::services::agent_local::agent_loop;
use crate::services::agent_local::chat_prompts::{
    prepend_tool_system_prompt, prepend_working_dir_context,
};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
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
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    let cancel = CancellationToken::new();
    streams.0.lock().await.insert(session_id.clone(), cancel.clone());

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
            cancel,
        )
        .await;

        task_app
            .state::<ActiveStreams>()
            .0
            .lock()
            .await
            .remove(&stream_session);

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

    if provider == "ollama" {
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
