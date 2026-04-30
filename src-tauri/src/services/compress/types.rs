use crate::services::agent_local::types_ollama::ChatMessage;

pub struct CompressionConfig {
    pub enabled: bool,
    pub threshold_pct: u8,
    pub configured_context_window: u64,
    pub native_context_window: u64,
}

pub struct CompressionResult {
    pub summary_message: ChatMessage,
    pub boundary_marker: ChatMessage,
    pub pre_compression_tokens: usize,
    pub post_compression_tokens: usize,
}
