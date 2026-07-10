use crate::services::agent_local::stream_events::AgentEventEmitter;
use std::sync::OnceLock;
use tauri::AppHandle;
use tokio::sync::mpsc;

pub struct SpawnRequest {
    pub app: AppHandle,
    pub parent_session_id: String,
    pub child_session_id: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub subagent_type: String,
    pub parent_emitter: AgentEventEmitter,
    pub cancel: tokio_util::sync::CancellationToken,
    pub project_id: Option<String>,
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
            let parent_session_id = req.parent_session_id.clone();
            let child_session_id = req.child_session_id.clone();
            let subagent_type = req.subagent_type.clone();
            let parent_emitter = req.parent_emitter.clone();
            let run_id = super::subagent_registry::get_run_id_for_child(&child_session_id).await;
            let child = super::subagent_task::run(
                req.app,
                req.parent_session_id,
                req.child_session_id,
                req.model,
                req.provider,
                req.prompt,
                req.subagent_type,
                req.parent_emitter,
                req.cancel,
                req.project_id,
            );
            super::subagent_panic_supervisor::run_guarded(child, move || async move {
                if super::subagent_panic_supervisor::recover_panicked_completion(
                    &parent_session_id,
                    &child_session_id,
                    &subagent_type,
                )
                .await
                {
                    let _ = parent_emitter.send(
                        crate::services::agent_local::types_ollama::StreamEvent::SubagentCompleted {
                            subagent_session_id: child_session_id,
                            success: false,
                            status: super::subagent_status::FAILED.to_string(),
                            summary: super::subagent_panic_supervisor::SUBAGENT_PANIC_SUMMARY
                                .to_string(),
                            run_id,
                        },
                    );
                }
            })
            .await;
        });
    }
}
