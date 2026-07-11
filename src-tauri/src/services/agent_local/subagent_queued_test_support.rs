use super::{session_store, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

pub async fn queued_child() -> (
    super::types_session::AgentSession,
    super::types_session::AgentSession,
) {
    let parent = session_store::create_full("Parent queued error", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    child.subagent_queued_prompts.push("Continue".into());
    session_store::save(&child).await.expect("save child");
    subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    (parent, child)
}
