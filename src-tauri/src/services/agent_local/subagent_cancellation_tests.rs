use super::{session_store, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn stale_cancel_cannot_touch_replacement_with_same_parent_run() {
    let parent = session("Parent", None).await;
    let mut child = session("Child", Some(&parent.id)).await;
    let sibling = session("Sibling", Some(&parent.id)).await;
    let old = register(&parent.id, &child.id).await;
    let _sibling = register(&parent.id, &sibling.id).await;
    child.subagent_run_id = Some(old.run_id.clone());
    session_store::save(&child).await.expect("save old run");
    subagent_registry::unregister(&child.id).await;
    let replacement = register(&parent.id, &child.id).await;
    child.subagent_run_id = Some(replacement.run_id.clone());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save replacement");

    assert_eq!(old.run_id, replacement.run_id);
    assert!(!subagent_registry::cancel_execution(&child.id, &old.execution_id).await);
    let active = subagent_registry::active_run_for_child(&child.id)
        .await
        .expect("replacement active");
    assert!(!active.cancelled);
    assert_eq!(active.execution_id, replacement.execution_id);
    assert_eq!(
        session_store::get(&child.id)
            .await
            .expect("load child")
            .subagent_status
            .as_deref(),
        Some(subagent_status::RUNNING)
    );

    cleanup(&[&child.id, &sibling.id, &parent.id]).await;
}

#[tokio::test]
async fn duplicate_registration_never_replaces_active_execution() {
    let parent = session("Parent duplicate", None).await;
    let child = session("Child duplicate", Some(&parent.id)).await;
    let first = register(&parent.id, &child.id).await;

    let duplicate = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await;

    assert!(duplicate.is_err());
    assert!(subagent_registry::owns_execution(
        &child.id,
        &first.run_id,
        &first.execution_id,
    )
    .await);
    cleanup(&[&child.id, &parent.id]).await;
}

async fn session(
    name: &str,
    parent_id: Option<&str>,
) -> super::types_session::AgentSession {
    let mut session = session_store::create_full(name, "llama3", "ollama", false, None)
        .await
        .expect("create session");
    session.parent_session_id = parent_id.map(ToString::to_string);
    if parent_id.is_some() {
        session.subagent_type = Some("explorer".into());
        session.subagent_status = Some(subagent_status::RUNNING.into());
    }
    session_store::save(&session).await.expect("save session");
    session
}

async fn register(parent_id: &str, child_id: &str) -> subagent_registry::RegisteredSubagent {
    subagent_registry::register_execution(parent_id, child_id, CancellationToken::new())
        .await
        .expect("register execution")
}

async fn cleanup(ids: &[&str]) {
    for id in ids {
        subagent_registry::unregister(id).await;
        session_store::delete_one(id).await.expect("delete session");
    }
}
