use super::subagent_completion::{persist_terminal_completion, SUBAGENT_COMPLETION_ERROR};
use super::{session_store, subagent_hidden_reports, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

async fn session(name: &str) -> super::types_session::AgentSession {
    session_store::create_full(name, "llama3", "ollama", false, None)
        .await
        .expect("create session")
}

async fn child_session(parent_id: &str) -> super::types_session::AgentSession {
    let mut child = session("Geminitor").await;
    child.parent_session_id = Some(parent_id.to_string());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    child
}

#[tokio::test]
async fn report_is_persistent_before_child_disappears_and_signal_fires() {
    let parent = session("Parent completion").await;
    let child = child_session(&parent.id).await;
    subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    let mut signal = subagent_registry::subscribe_for_parent(&parent.id)
        .await
        .expect("subscribe signal");

    persist_terminal_completion(
        &parent.id,
        &child.id,
        "explorer",
        subagent_status::COMPLETED,
        "Rapport final",
    )
    .await
    .expect("persist completion");

    signal.changed().await.expect("terminal signal");
    assert_eq!(signal.borrow().sequence, 1);
    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    let reports = subagent_hidden_reports::peek_reports(&parent.id).await;
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].summary, "Rapport final");
    assert_eq!(
        session_store::get(&child.id)
            .await
            .expect("saved child")
            .subagent_status
            .as_deref(),
        Some(subagent_status::COMPLETED)
    );
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn report_persistence_failure_is_terminal_generic_and_not_silent() {
    let missing_parent = uuid::Uuid::new_v4().to_string();
    let child = child_session(&missing_parent).await;
    subagent_registry::register(&missing_parent, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    let mut signal = subagent_registry::subscribe_for_parent(&missing_parent)
        .await
        .expect("subscribe signal");

    let error = match persist_terminal_completion(
        &missing_parent,
        &child.id,
        "explorer",
        subagent_status::COMPLETED,
        "Rapport final",
    )
    .await
    {
        Ok(_) => panic!("parent persistence must fail"),
        Err(error) => error,
    };

    assert_eq!(error, SUBAGENT_COMPLETION_ERROR);
    signal.changed().await.expect("failure signal");
    assert!(signal.borrow().report_persistence_failed);
    assert!(
        subagent_registry::active_children_for_parent(&missing_parent)
            .await
            .is_empty()
    );
    assert_eq!(
        session_store::get(&child.id)
            .await
            .expect("saved child")
            .subagent_status
            .as_deref(),
        Some(subagent_status::FAILED)
    );
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
}
