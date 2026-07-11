use crate::services::agent_local::types_ollama::StreamEvent;
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter};

pub const AGENT_STREAM_EVENT: &str = "agent-stream-event";

pub fn next_generation() -> u64 {
    crate::STREAM_GENERATION.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub struct AgentEventEmitter {
    app: AppHandle,
    session_id: String,
    generation: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamEventPayload {
    session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation: Option<u64>,
    event: StreamEvent,
}

impl AgentEventEmitter {
    pub fn new(app: AppHandle, session_id: String) -> Self {
        Self {
            app,
            session_id,
            generation: None,
        }
    }

    pub fn with_generation(app: AppHandle, session_id: String, generation: u64) -> Self {
        Self {
            app,
            session_id,
            generation: Some(generation),
        }
    }

    pub fn send(&self, event: StreamEvent) -> Result<(), String> {
        self.app
            .emit(
                AGENT_STREAM_EVENT,
                StreamEventPayload {
                    session_id: self.session_id.clone(),
                    generation: self.generation,
                    event,
                },
            )
            .map_err(|_| "Emission evenement impossible".to_string())
    }
}
