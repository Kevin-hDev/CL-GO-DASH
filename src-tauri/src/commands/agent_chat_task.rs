use crate::services::agent_local::agent_loop;
use crate::services::agent_local::agent_md;
use crate::services::agent_local::agent_settings;
use crate::services::agent_local::chat_prompts::prepare_messages;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::git_context::{self, GitSnapshot};
use crate::services::llm;
use crate::services::personality_injection;
use tokio_util::sync::CancellationToken;

fn permission_level(mode: &str) -> u8 {
    match mode {
        "manual" => 0,
        "chat" => 1,
        "subagent" => 2,
        "auto" => 3,
        _ => 0,
    }
}

fn is_more_permissive(requested: &str, stored: &str) -> bool {
    permission_level(requested) > permission_level(stored)
}

pub(crate) fn merge_personality(
    agent_md: Option<String>,
    personality: Option<String>,
) -> Option<String> {
    match (agent_md, personality) {
        (Some(a), Some(p)) => Some(format!("{a}\n\n{p}")),
        (Some(a), None) => Some(a),
        (None, Some(p)) => Some(p),
        (None, None) => None,
    }
}

async fn collect_git_snapshot(working_dir: &std::path::Path) -> GitSnapshot {
    let wd = working_dir.to_path_buf();
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::task::spawn_blocking(move || git_context::detect_git(&wd)),
    )
    .await
    .ok()
    .and_then(|r| r.ok())
    .unwrap_or_default()
}

fn append_git_section(messages: &mut Vec<ChatMessage>, snap: &GitSnapshot) {
    if let Some(section) = git_context::format_git_section(snap) {
        if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
            first.content.push_str(&format!("\n\n{section}"));
        }
    }
}

fn is_compress_command(messages: &[ChatMessage]) -> bool {
    messages
        .last()
        .map(|m| m.role == "user" && m.content.trim() == "/compress")
        .unwrap_or(false)
}

async fn handle_compress_command(
    on_event: &AgentEventEmitter,
    session_id: &str,
    messages: &[ChatMessage],
    model: &str,
    provider: &str,
    cancel: CancellationToken,
) -> Result<(), String> {
    use crate::services::agent_local::session_store;
    use crate::services::agent_local::types_ollama::StreamEvent;
    use crate::services::agent_local::types_session::AgentMessage;
    use crate::services::compress::{engine, prompt};

    let _ = on_event.send(StreamEvent::Compressing {
        status: "start".to_string(),
    });

    let msgs_without_command: Vec<ChatMessage> = messages
        .iter()
        .filter(|m| !(m.role == "user" && m.content.trim() == "/compress"))
        .cloned()
        .collect();

    let compress_msgs = engine::build_compression_request_content(&msgs_without_command, None);

    let summary_raw = if provider == "ollama" {
        crate::services::agent_local::ollama_stream::collect_chat(model, compress_msgs)
            .await
            .map(|(content, _)| content)
            .map_err(|e| format!("Compression Ollama : {e}"))?
    } else {
        crate::services::llm::stream::collect_chat_silent(provider, model, &compress_msgs, cancel)
            .await
            .map(|r| r.content)
            .map_err(|e| format!("Compression LLM : {e}"))?
    };

    let summary = prompt::extract_summary(&summary_raw);
    let summary_content = prompt::format_summary_message(&summary, false);

    // Estimer les tokens du résumé
    let summary_chat_msg = ChatMessage {
        role: "assistant".to_string(),
        content: summary_content.clone(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    };
    let summary_tokens =
        crate::services::compress::token_estimate::estimate_tokens(&[summary_chat_msg]);

    let compressed_msg = AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "assistant".to_string(),
        content: summary_content,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: summary_tokens as u32,
        skill_names: None,
    };

    if let Ok(mut session) = session_store::get(session_id).await {
        session.messages = vec![compressed_msg];
        session.accumulated_tokens = summary_tokens as u32;
        let _ = session_store::save(&session).await;
    }

    let _ = on_event.send(StreamEvent::Compressing {
        status: "done".to_string(),
    });
    let _ = on_event.send(StreamEvent::CompressionComplete {});
    let _ = on_event.send(StreamEvent::Done {
        eval_count: 0,
        eval_duration_ns: 0,
        final_tps: 0.0,
        prompt_tokens: 0,
        context_tokens: summary_tokens as u32,
    });

    Ok(())
}

pub(crate) async fn run_stream_task(
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
    reasoning_mode: Option<String>,
    permission_mode_override: Option<String>,
    cancel: CancellationToken,
) -> Result<Vec<ChatMessage>, String> {
    use crate::services::agent_local::session_store;

    let resolve_dir =
        |wd: &Option<String>, _session_id: &str| -> Result<std::path::PathBuf, String> {
            if let Some(d) = wd.as_ref().filter(|s| !s.is_empty()) {
                let p = std::path::PathBuf::from(d);
                if p.is_dir() {
                    return p.canonicalize().map_err(|e| {
                        eprintln!("[agent] canonicalize dir: {e}");
                        "Répertoire inaccessible".to_string()
                    });
                }
                return Err(format!("Répertoire introuvable : {d}"));
            }
            Ok(dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap()))
        };

    // Interception : /compress déclenche la compression manuelle
    if is_compress_command(&messages) {
        handle_compress_command(&on_event, &session_id, &messages, &model, &provider, cancel)
            .await?;
        return Ok(messages);
    }

    let mode = {
        let stored = agent_settings::get_permission_mode().await;
        match permission_mode_override {
            Some(m) if matches!(m.as_str(), "auto" | "manual" | "chat" | "subagent") => {
                if is_more_permissive(&m, &stored) {
                    stored
                } else {
                    m
                }
            }
            _ => stored,
        }
    };
    let is_chat = mode == "chat";
    let is_subagent = mode == "subagent";
    let response_language = crate::services::config::read_config()
        .map(|c| c.advanced.response_language)
        .unwrap_or_default();

    if provider == "ollama" {
        let ctx = crate::services::compress::context_resolve::resolve_ollama(&model).await;

        let final_tools = if tools.is_empty() {
            if is_chat {
                tool_dispatcher::get_chat_tool_definitions()
            } else {
                tool_dispatcher::get_tool_definitions()
            }
        } else {
            tools
        };

        let working_dir = resolve_dir(&working_dir, &session_id)?;
        let _ =
            session_store::update_working_dir(&session_id, &working_dir.to_string_lossy()).await;
        let snap = collect_git_snapshot(&working_dir).await;
        let mut msgs = messages;
        let agent_md_content = if is_chat || is_subagent {
            None
        } else {
            let raw = agent_md::load_agent_md(Some(working_dir.as_path())).await;
            let personality = personality_injection::load_injected_contents();
            merge_personality(raw, personality)
        };
        let skills_tuples: Vec<(String, String)> = if is_chat || is_subagent {
            vec![]
        } else {
            tool_skill_loader::list_skills()
                .await
                .unwrap_or_default()
                .iter()
                .map(|s| (s.name.clone(), s.description.clone()))
                .collect()
        };
        prepare_messages(
            &mut msgs,
            &working_dir,
            snap.is_git,
            snap.git_root.as_deref(),
            true,
            agent_md_content,
            &skills_tuples,
            &model,
            &mode,
            &response_language,
        );
        append_git_section(&mut msgs, &snap);

        let ollama_think = crate::services::reasoning::ollama_think(
            &model,
            reasoning_mode.as_deref(),
            think,
        )
        .unwrap_or(crate::services::agent_local::types_ollama::OllamaThink::Bool(false));
        agent_loop::run_agent_loop(
            &on_event,
            &mut msgs,
            &model,
            final_tools,
            ollama_think,
            working_dir,
            session_id.clone(),
            cancel,
            ctx.native,
            ctx.configured,
            &mode,
        )
        .await?;
        Ok(msgs)
    } else {
        use crate::services::llm::{model_registry, tool_capable};
        let ctx = crate::services::compress::context_resolve::resolve_api(&provider, &model).await;
        let registry_caps = model_registry::lookup(&provider, &model).await;
        let model_supports_tools = supports_tools_hint.unwrap_or_else(|| {
            registry_caps
                .as_ref()
                .map(|c| c.supports_tools)
                .unwrap_or(false)
                || tool_capable::supports_tools(&provider, &model)
        });
        let model_supports_thinking = supports_thinking_hint.unwrap_or_else(|| {
            registry_caps
                .as_ref()
                .map(|c| c.supports_thinking)
                .unwrap_or(false)
                || tool_capable::supports_thinking(&provider, &model)
        });

        let model_supports_vision = registry_caps
            .as_ref()
            .map(|c| c.supports_vision)
            .unwrap_or(false)
            || tool_capable::supports_vision(&provider, &model);

        let final_tools = if is_chat {
            tool_dispatcher::get_chat_tool_definitions()
        } else if model_supports_tools {
            if tools.is_empty() {
                tool_dispatcher::get_tool_definitions()
            } else {
                tools
            }
        } else {
            vec![]
        };
        let openai_tools = llm::agent_loop::convert_tools_to_openai(&final_tools);
        let working_dir = resolve_dir(&working_dir, &session_id)?;
        let _ =
            session_store::update_working_dir(&session_id, &working_dir.to_string_lossy()).await;
        let snap = collect_git_snapshot(&working_dir).await;
        let mut msgs = messages;
        if !model_supports_vision {
            llm::stream_convert::strip_images(&mut msgs);
        }
        let agent_md_content = if is_chat || is_subagent {
            None
        } else {
            let raw = agent_md::load_agent_md(Some(working_dir.as_path())).await;
            let personality = personality_injection::load_injected_contents();
            merge_personality(raw, personality)
        };
        let skills_tuples: Vec<(String, String)> =
            if is_chat || is_subagent || !model_supports_tools {
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
        prepare_messages(
            &mut msgs,
            &working_dir,
            snap.is_git,
            snap.git_root.as_deref(),
            has_tools,
            agent_md_content,
            &skills_tuples,
            &model,
            &mode,
            &response_language,
        );
        append_git_section(&mut msgs, &snap);
        let think_active = crate::services::reasoning::enabled(reasoning_mode.as_deref(), think)
            && model_supports_thinking;
        llm::agent_loop::run_agent_loop(
            &on_event,
            &provider,
            &model,
            &mut msgs,
            &openai_tools,
            think_active,
            reasoning_mode.as_deref(),
            working_dir,
            session_id.clone(),
            cancel,
            ctx.native,
            ctx.configured,
            &mode,
        )
        .await?;
        Ok(msgs)
    }
}
