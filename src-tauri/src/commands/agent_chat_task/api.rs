use super::common::{self, StreamMode};
use super::params::StreamTaskParams;
use crate::services::agent_local::{agent_settings::AgentSettings, tool_catalog, tool_dispatcher};
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::llm;
use crate::services::llm::{model_registry, tool_capable};

struct ApiCapabilities {
    tools: bool,
    thinking: bool,
    vision: bool,
}

pub(crate) async fn run(
    params: StreamTaskParams,
    mode: StreamMode,
    response_language: String,
) -> Result<Vec<ChatMessage>, String> {
    let ctx =
        crate::services::compress::context_resolve::resolve_api(&params.provider, &params.model)
            .await;
    let caps = resolve_capabilities(&params).await;
    let settings = crate::services::agent_local::agent_settings::load().await;
    let final_tools = resolve_tools(&params, &mode, caps.tools, &settings);
    let enabled_tool_names = tool_catalog::tool_names(&final_tools);
    let openai_tools = llm::agent_loop_tools::convert_tools_to_openai(&final_tools);
    let working_dir = common::resolve_working_dir(&params.working_dir)?;
    common::update_working_dir(&params.session_id, &working_dir).await;
    let plan_mode_active =
        resolve_plan_mode(&params).await && tool_catalog::has_plan_tools(&enabled_tool_names);

    let snap = common::collect_git_snapshot(&working_dir).await;
    let agent_md = common::agent_md_content(&mode, &working_dir).await;
    let has_tools = !final_tools.is_empty();
    let skills = common::skills_tuples(
        !mode.is_chat
            && !mode.is_subagent
            && has_tools
            && tool_catalog::has_tool(&enabled_tool_names, "load_skill"),
    )
    .await;
    let mut messages = params.messages;
    sanitize_images(&params.on_event, &mut messages, caps.vision);
    common::prepare_with_context(
        &mut messages,
        common::PromptContext {
            working_dir: &working_dir,
            snap: &snap,
            has_tools,
            agent_md_content: agent_md,
            skills: &skills,
            model: &params.model,
            mode: &mode.mode,
            response_language: &response_language,
            plan_mode_active,
            enabled_tool_names: &enabled_tool_names,
        },
    );
    if todo_tools_enabled(&enabled_tool_names) {
        crate::services::agent_local::tool_todo::append_session_reminder(
            &mut messages,
            &params.session_id,
        )
        .await;
    }
    super::gemma4_thinking_guard::apply(&mut messages, &params.provider, &params.model);

    let effective_reasoning_mode = crate::services::reasoning::normalize_for_model(
        &params.provider,
        &params.model,
        params.reasoning_mode.as_deref(),
        caps.thinking,
    );
    let think_active =
        crate::services::reasoning::enabled(effective_reasoning_mode.as_deref(), params.think)
            && caps.thinking;
    llm::agent_loop::run_agent_loop(
        &params.on_event,
        &params.provider,
        &params.model,
        &mut messages,
        &openai_tools,
        think_active,
        effective_reasoning_mode.as_deref(),
        working_dir,
        params.session_id.clone(),
        params.request_id.clone(),
        params.cancel,
        ctx.native,
        ctx.configured,
        &mode.mode,
        plan_mode_active,
    )
    .await?;
    Ok(messages)
}

async fn resolve_plan_mode(params: &StreamTaskParams) -> bool {
    match params.plan_mode {
        Some(value) => value,
        None => crate::services::agent_local::tool_plan::is_enabled(&params.session_id).await,
    }
}

async fn resolve_capabilities(params: &StreamTaskParams) -> ApiCapabilities {
    let registry_caps = model_registry::lookup(&params.provider, &params.model).await;
    ApiCapabilities {
        tools: params.capability_hints.supports_tools.unwrap_or_else(|| {
            registry_caps
                .as_ref()
                .map(|c| c.supports_tools)
                .unwrap_or(false)
                || tool_capable::supports_tools(&params.provider, &params.model)
        }),
        thinking: params
            .capability_hints
            .supports_thinking
            .unwrap_or_else(|| {
                params.provider == "codex-oauth"
                    || registry_caps
                        .as_ref()
                        .map(|c| c.supports_thinking)
                        .unwrap_or(false)
                    || tool_capable::supports_thinking(&params.provider, &params.model)
            }),
        vision: params.capability_hints.supports_vision.unwrap_or_else(|| {
            registry_caps
                .as_ref()
                .map(|c| c.supports_vision)
                .unwrap_or(false)
                || params.provider == "codex-oauth"
                || tool_capable::supports_vision(&params.provider, &params.model)
        }),
    }
}

fn resolve_tools(
    params: &StreamTaskParams,
    mode: &StreamMode,
    model_supports_tools: bool,
    settings: &AgentSettings,
) -> Vec<serde_json::Value> {
    let defs = if mode.is_chat {
        tool_dispatcher::get_chat_tool_definitions()
    } else if !model_supports_tools {
        vec![]
    } else if params.tools.is_empty() {
        tool_dispatcher::get_tool_definitions()
    } else {
        params.tools.clone()
    };
    tool_catalog::filter_tool_definitions(defs, &settings.enabled_optional_tools)
}

fn todo_tools_enabled(enabled_tool_names: &[String]) -> bool {
    tool_catalog::has_any_tool(
        enabled_tool_names,
        &[
            "todo_write",
            "todo_history",
            "todo_pause",
            "todo_resume",
            "todo_delete",
            "agent_diagnostics",
        ],
    )
}

fn sanitize_images(
    on_event: &crate::services::agent_local::stream_events::AgentEventEmitter,
    messages: &mut [ChatMessage],
    supports_vision: bool,
) {
    let image_report = crate::services::llm::vision::sanitize_messages(messages, supports_vision);
    if image_report.unsupported_removed > 0 {
        let _ = on_event.send(StreamEvent::Notice {
            message_key: crate::services::llm::vision::NOTICE_UNSUPPORTED_MODEL.to_string(),
        });
    } else if image_report.invalid_removed > 0 {
        let _ = on_event.send(StreamEvent::Notice {
            message_key: crate::services::llm::vision::NOTICE_IMAGE_SKIPPED.to_string(),
        });
    }
}
