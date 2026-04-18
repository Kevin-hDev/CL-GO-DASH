use crate::services::agent_local::types_ollama::StreamEvent;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub const AGENT_STREAM_EVENT: &str = "agent-stream-event";

#[derive(Clone)]
pub struct AgentEventEmitter {
    app: AppHandle,
    session_id: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamEventPayload {
    session_id: String,
    event: StreamEvent,
}

impl AgentEventEmitter {
    pub fn new(app: AppHandle, session_id: String) -> Self {
        Self { app, session_id }
    }

    pub fn send(&self, event: StreamEvent) -> Result<(), String> {
        self.app
            .emit(
                AGENT_STREAM_EVENT,
                StreamEventPayload {
                    session_id: self.session_id.clone(),
                    event,
                },
            )
            .map_err(|_| "Emission evenement impossible".to_string())
    }
}
