use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use tokio::sync::oneshot;

pub struct PendingDelegate {
    child_id: String,
    receiver: oneshot::Receiver<super::subagent_completion::SubagentCompletion>,
}

pub async fn dispatch_delegate(args: &Value, session_id: &str) -> ToolResult {
    match spawn_delegate(args, session_id).await {
        Ok(pending) => pending.wait().await,
        Err(tr) => tr,
    }
}

pub async fn spawn_delegate(args: &Value, session_id: &str) -> Result<PendingDelegate, ToolResult> {
    let Some(app) = super::app_handle_global::get() else {
        return Err(ToolResult::err("AppHandle non initialisé"));
    };
    let emitter = crate::services::agent_local::stream_events::AgentEventEmitter::new(
        app.clone(),
        session_id.to_string(),
    );
    match super::tool_delegate::prepare_delegate(
        args.clone(),
        app.clone(),
        session_id.to_string(),
        emitter,
    )
    .await
    {
        Err(tr) => Err(tr),
        Ok(spawned) => {
            let child_id = spawned.child_id.clone();
            let (tx, rx) = oneshot::channel();
            if let Err(e) =
                super::subagent_spawn_channel::send(super::subagent_spawn_channel::SpawnRequest {
                    app: spawned.app,
                    child_session_id: spawned.child_id,
                    model: spawned.model,
                    provider: spawned.provider,
                    prompt: spawned.prompt,
                    subagent_type: spawned.subagent_type,
                    parent_emitter: spawned.parent_emitter,
                    cancel: spawned.cancel,
                    project_id: spawned.project_id,
                    completion_tx: tx,
                })
            {
                super::subagent_registry::unregister(&child_id).await;
                if let Err(mark_err) =
                    super::session_subagents::mark_status(&child_id, super::subagent_status::FAILED)
                        .await
                {
                    eprintln!("[delegate] mark_status failed {child_id}: {mark_err}");
                }
                return Err(ToolResult::err(e));
            }
            Ok(PendingDelegate {
                child_id,
                receiver: rx,
            })
        }
    }
}

impl PendingDelegate {
    pub async fn wait(self) -> ToolResult {
        match self.receiver.await {
            Ok(completion) => completion.to_tool_result(),
            Err(_) => {
                super::subagent_registry::unregister(&self.child_id).await;
                if let Err(e) = super::session_subagents::mark_status(
                    &self.child_id,
                    super::subagent_status::FAILED,
                )
                .await
                {
                    eprintln!("[delegate] mark_status failed {}: {e}", self.child_id);
                }
                ToolResult::err("Le sous-agent n'a pas pu terminer correctement.")
            }
        }
    }
}
