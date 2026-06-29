mod api;
mod common;
mod compress;
mod gemma4_thinking_guard;
mod ollama;
mod params;

pub(crate) use params::{StreamCapabilityHints, StreamTaskParams};

use crate::services::agent_local::types_ollama::ChatMessage;

pub(crate) use common::merge_personality;

pub(crate) async fn run_stream_task(params: StreamTaskParams) -> Result<Vec<ChatMessage>, String> {
    if compress::is_compress_command(&params.messages) {
        let working_dir = common::resolve_working_dir(&params.working_dir)?;
        common::update_working_dir(&params.session_id, &working_dir).await;
        compress::handle_compress_command(
            &params.on_event,
            &params.session_id,
            &params.request_id,
            &params.messages,
            &params.model,
            &params.provider,
            &working_dir,
            params.cancel.clone(),
        )
        .await?;
        return Ok(params.messages);
    }

    let mode = common::resolve_permission_mode(params.permission_mode_override.as_deref()).await;
    let response_language = common::response_language();

    if params.provider == "ollama" {
        ollama::run(params, mode, response_language).await
    } else {
        api::run(params, mode, response_language).await
    }
}
