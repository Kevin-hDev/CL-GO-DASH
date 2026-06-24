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
        compress::handle_compress_command(
            &params.on_event,
            &params.session_id,
            &params.messages,
            &params.model,
            &params.provider,
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
