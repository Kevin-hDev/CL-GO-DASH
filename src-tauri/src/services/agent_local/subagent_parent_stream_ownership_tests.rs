use super::subagent_registry;
use tokio_util::sync::CancellationToken;

fn uid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tokio::test]
async fn replacement_adopts_existing_children_and_current_stop_cancels_them() {
    let parent = uid();
    let adopted_child = uid();
    let new_child = uid();
    let old_owner = CancellationToken::new();
    let new_owner = CancellationToken::new();
    subagent_registry::register_execution_for_parent_stream(
        &parent,
        &adopted_child,
        CancellationToken::new(),
        None,
        &old_owner,
    )
    .await
    .expect("register old child");
    old_owner.cancel();
    subagent_registry::adopt_children_for_parent_stream(&parent, &new_owner).await;
    subagent_registry::register_execution_for_parent_stream(
        &parent,
        &new_child,
        CancellationToken::new(),
        None,
        &new_owner,
    )
    .await
    .expect("register new child");

    subagent_registry::cancel_stopped_parent_stream_children(&parent).await;
    assert!(!run(&adopted_child).await.cancelled);
    assert!(!run(&new_child).await.cancelled);

    new_owner.cancel();
    subagent_registry::cancel_stopped_parent_stream_children(&parent).await;
    assert!(run(&adopted_child).await.cancelled);
    assert!(run(&new_child).await.cancelled);
    subagent_registry::unregister(&adopted_child).await;
    subagent_registry::unregister(&new_child).await;
}

#[test]
fn chat_stream_transfers_and_cancels_parent_stream_ownership() {
    let source = include_str!("../../commands/agent_chat.rs");
    let adopt = source
        .find("adopt_children_for_parent_stream")
        .expect("chat stream adopts children");
    let start = source
        .find("stream_diagnostics::start_request")
        .expect("chat stream starts request");

    assert!(adopt < start);
    assert!(source.contains("cancel_stopped_parent_stream_children"));
}

async fn run(child_id: &str) -> subagent_registry::ActiveSubagentRun {
    subagent_registry::active_run_for_child(child_id)
        .await
        .expect("active child")
}
