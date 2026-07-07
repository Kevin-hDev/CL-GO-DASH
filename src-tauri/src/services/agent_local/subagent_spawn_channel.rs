use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_completion::SubagentCompletion;
use std::sync::OnceLock;
use tauri::AppHandle;
use tokio::sync::{mpsc, oneshot};

pub struct SpawnRequest {
    pub app: AppHandle,
    pub child_session_id: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub subagent_type: String,
    pub parent_emitter: AgentEventEmitter,
    pub cancel: tokio_util::sync::CancellationToken,
    pub project_id: Option<String>,
    pub completion_tx: Option<oneshot::Sender<SubagentCompletion>>,
}

const MAX_QUEUED: usize = 8;

static TX: OnceLock<mpsc::Sender<SpawnRequest>> = OnceLock::new();

pub fn init() {
    let (tx, rx) = mpsc::channel(MAX_QUEUED);
    let _ = TX.set(tx);
    tauri::async_runtime::spawn(receiver_loop(rx));
}

pub fn send(req: SpawnRequest) -> Result<(), String> {
    TX.get()
        .ok_or_else(|| "Canal de spawn non initialisé".to_string())?
        .try_send(req)
        .map_err(|_| "Trop de sous-agents en attente".to_string())
}

async fn receiver_loop(mut rx: mpsc::Receiver<SpawnRequest>) {
    while let Some(req) = rx.recv().await {
        tauri::async_runtime::spawn(async move {
            super::subagent_task::run(
                req.app,
                req.child_session_id,
                req.model,
                req.provider,
                req.prompt,
                req.subagent_type,
                req.parent_emitter,
                req.cancel,
                req.project_id,
                req.completion_tx,
            )
            .await;
        });
    }
}
