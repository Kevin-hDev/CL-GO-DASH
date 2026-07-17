use crate::services::acp::{native_tool_allowed, AcpUpdate};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::oauth_providers::ProviderId;

const MAX_TOOL_CALLS: usize = 128;
const MAX_AUX_ACTIVITIES: usize = 32;
const MAX_STREAM_TEXT_BYTES: usize = 8 * 1024 * 1024;

struct ToolState {
    id: String,
    stable_id: String,
    name: String,
    index: usize,
    source: String,
    kind: Option<String>,
}

pub struct AcpTurnState {
    tools: Vec<ToolState>,
    token_count: u32,
    text_bytes: usize,
    session_id: String,
    aux_count: usize,
}

impl AcpTurnState {
    pub fn new(session_id: &str) -> Self {
        Self {
            tools: Vec::new(),
            token_count: 0,
            text_bytes: 0,
            session_id: session_id.to_string(),
            aux_count: 0,
        }
    }

    pub fn token_count(&self) -> u32 {
        self.token_count
    }

    pub fn tool_name(&self, id: &str) -> Option<&str> {
        self.tools
            .iter()
            .find(|tool| tool.id == id)
            .map(|tool| tool.name.as_str())
    }

    pub fn handle(
        &mut self,
        provider: ProviderId,
        update: AcpUpdate,
        on_event: &AgentEventEmitter,
    ) -> Result<(), String> {
        match update {
            AcpUpdate::Text(content) => self.emit_text(content, on_event),
            AcpUpdate::Thought(content) => {
                let _ = on_event.send(StreamEvent::Thinking { content });
                Ok(())
            }
            AcpUpdate::ToolCall {
                id,
                name,
                arguments,
                source,
                kind,
            } => {
                if source == "native" && !native_tool_allowed(provider, &name) {
                    return Err("Outil natif interdit".to_string());
                }
                if self.tools.len() >= MAX_TOOL_CALLS {
                    return Err("Trop d'outils ACP".to_string());
                }
                let index = self.tools.len();
                let provider_id = provider_name(provider);
                let stable_id = format!("{}:{provider_id}:{source}:{id}", self.session_id);
                self.tools.push(ToolState {
                    id,
                    stable_id: stable_id.clone(),
                    name: name.clone(),
                    index,
                    source: source.clone(),
                    kind: kind.clone(),
                });
                let _ = on_event.send(StreamEvent::ProviderToolCall {
                    name,
                    arguments,
                    tool_call_id: stable_id,
                    provider_id: provider_id.to_string(),
                    source,
                    status: "pending".to_string(),
                    kind,
                });
                Ok(())
            }
            AcpUpdate::ToolUpdate {
                id,
                status,
                content,
            } => {
                if let Some(tool) = self.tools.iter().find(|tool| tool.id == id) {
                    let status = status.unwrap_or_else(|| "in_progress".to_string());
                    let _ = on_event.send(StreamEvent::ProviderToolResult {
                        name: tool.name.clone(),
                        content: content.unwrap_or_default(),
                        is_error: status == "failed",
                        truncated: false,
                        tool_call_index: tool.index,
                        tool_call_id: tool.stable_id.clone(),
                        provider_id: provider_name(provider).to_string(),
                        source: tool.source.clone(),
                        status,
                        kind: tool.kind.clone(),
                    });
                }
                Ok(())
            }
            AcpUpdate::Plan(plan) => self.emit_aux(provider, "agent_plan", plan, on_event),
            AcpUpdate::Ignored => Ok(()),
            AcpUpdate::Unknown(kind) => self.emit_aux(
                provider,
                "acp_activity",
                serde_json::json!({"event":kind}),
                on_event,
            ),
        }
    }

    fn emit_text(&mut self, content: String, on_event: &AgentEventEmitter) -> Result<(), String> {
        self.text_bytes = self.text_bytes.saturating_add(content.len());
        if self.text_bytes > MAX_STREAM_TEXT_BYTES {
            return Err("Réponse ACP trop grande".to_string());
        }
        self.token_count = self.token_count.saturating_add(1);
        let _ = on_event.send(StreamEvent::Token {
            content,
            token_count: self.token_count,
            tps: 0.0,
            phase: None,
        });
        Ok(())
    }

    fn emit_aux(
        &mut self,
        provider: ProviderId,
        name: &str,
        arguments: serde_json::Value,
        on_event: &AgentEventEmitter,
    ) -> Result<(), String> {
        if self.aux_count >= MAX_AUX_ACTIVITIES {
            return Ok(());
        }
        let index = self.tools.len();
        let provider_id = provider_name(provider);
        let stable_id = format!(
            "{}:{provider_id}:native:{name}:{}",
            self.session_id, self.aux_count
        );
        self.aux_count += 1;
        let _ = on_event.send(StreamEvent::ProviderToolCall {
            name: name.to_string(),
            arguments,
            tool_call_id: stable_id.clone(),
            provider_id: provider_id.to_string(),
            source: "native".to_string(),
            status: "completed".to_string(),
            kind: Some("other".to_string()),
        });
        let _ = on_event.send(StreamEvent::ProviderToolResult {
            name: name.to_string(),
            content: String::new(),
            is_error: false,
            truncated: false,
            tool_call_index: index,
            tool_call_id: stable_id,
            provider_id: provider_id.to_string(),
            source: "native".to_string(),
            status: "completed".to_string(),
            kind: Some("other".to_string()),
        });
        Ok(())
    }
}

fn provider_name(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::OpenAi => "openai",
        ProviderId::Moonshot => "moonshot",
        ProviderId::Xai => "xai",
    }
}
