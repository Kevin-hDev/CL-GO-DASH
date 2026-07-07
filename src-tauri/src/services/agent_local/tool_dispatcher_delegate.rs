use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;

pub async fn dispatch_delegate(args: &Value, session_id: &str) -> ToolResult {
    let Some(app) = super::app_handle_global::get() else {
        return ToolResult::err("AppHandle non initialisé");
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
        Err(tr) => tr,
        Ok(spawned) => {
            let msg = spawned.result_message.clone();
            let child_id = spawned.child_id.clone();
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
                })
            {
                super::subagent_registry::unregister(&child_id).await;
                if let Err(mark_err) =
                    super::session_subagents::mark_status(&child_id, "failed").await
                {
                    eprintln!("[delegate] mark_status failed {child_id}: {mark_err}");
                }
                return ToolResult::err(e);
            }
            ToolResult::ok(msg)
        }
    }
}
