use super::compress_hook;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::ChatMessage;
use crate::services::token_counting;
use tokio_util::sync::CancellationToken;

pub(super) struct LoopCompression<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub model: &'a str,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub native_context: u64,
    pub configured_context: u64,
}

impl LoopCompression<'_> {
    pub async fn try_run(
        &self,
        messages: &mut Vec<ChatMessage>,
        last_prompt: Option<u32>,
        last_eval: Option<u32>,
        cancel: CancellationToken,
    ) -> Option<u32> {
        compress_hook::try_auto_compress(
            self.on_event,
            messages,
            self.model,
            self.session_id,
            self.request_id,
            self.native_context,
            self.configured_context,
            token_counting::sum_real_counts(last_prompt, last_eval),
            cancel,
        )
        .await
    }
}
