use super::{
    session_store, subagent_completion, subagent_hidden_reports, subagent_registry,
    subagent_status,
};
use super::subagent_queued_test_support::queued_child;

#[tokio::test]
async fn missing_prompt_after_queued_completion_is_terminalized() {
    let (parent, child) = queued_child().await;
    let finalized = subagent_completion::persist_terminal_completion(
        &parent.id,
        &child.id,
        "explorer",
        subagent_status::COMPLETED,
        "Étape terminée",
    )
    .await
    .expect("persist queued completion");
    assert!(finalized.queued_followup);
    let mut saved = session_store::get(&child.id).await.expect("saved child");
    saved.subagent_queued_prompts.clear();
    session_store::save(&saved).await.expect("remove queued prompt");

    super::subagent_queued::finalize_unstarted_followup(&parent.id, &child.id, "explorer")
        .await
        .expect("terminalize missing follow-up");

    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    assert_eq!(
        session_store::get(&child.id)
            .await
            .expect("failed child")
            .subagent_status
            .as_deref(),
        Some(subagent_status::FAILED)
    );
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn has_next_prompt_error_terminalizes_registered_followup() {
    let (parent, child) = queued_child().await;
    session_store::delete_one(&child.id)
        .await
        .expect("delete child before has-next check");

    assert!(super::subagent_queued::prepare_next_followup(
        &parent.id,
        &child.id,
        "explorer",
    )
    .await
    .is_err());

    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    let reports = subagent_hidden_reports::peek_reports(&parent.id).await;
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].summary, subagent_completion::SUBAGENT_COMPLETION_ERROR);
    assert!(
        subagent_registry::terminal_state_for_parent(&parent.id)
            .await
            .expect("terminal failure signal")
            .report_persistence_failed
    );
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn renew_error_still_notifies_parent_and_persists_generic_report() {
    let (parent, child) = queued_child().await;
    subagent_registry::unregister(&child.id).await;

    assert!(super::subagent_queued::prepare_next_followup(
        &parent.id,
        &child.id,
        "explorer",
    )
    .await
    .is_err());

    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    assert_eq!(subagent_hidden_reports::peek_reports(&parent.id).await.len(), 1);
    assert_eq!(
        session_store::get(&child.id)
            .await
            .expect("failed child")
            .subagent_status
            .as_deref(),
        Some(subagent_status::FAILED)
    );
    assert_eq!(
        subagent_registry::terminal_state_for_parent(&parent.id)
            .await
            .expect("orphan follow-up terminal signal")
            .sequence,
        1
    );
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn take_prompt_error_after_renew_removes_active_entry() {
    let (parent, child) = queued_child().await;
    let deleted_child = child.id.clone();

    assert!(super::subagent_queued_transition::prepare_next_followup_with_after_renew(
        &parent.id,
        &child.id,
        "explorer",
        move || async move {
            session_store::delete_one(&deleted_child)
                .await
                .expect("delete child between renew and take");
        },
    )
    .await
    .is_err());

    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    assert_eq!(subagent_hidden_reports::peek_reports(&parent.id).await.len(), 1);
    assert!(
        subagent_registry::terminal_state_for_parent(&parent.id)
            .await
            .expect("take failure terminal signal")
            .report_persistence_failed
    );
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn spawn_send_error_terminalizes_prepared_followup() {
    let (parent, child) = queued_child().await;
    let prepared = super::subagent_queued::prepare_next_followup(
        &parent.id,
        &child.id,
        "explorer",
    )
    .await
    .expect("prepare follow-up")
    .expect("queued prompt");
    drop(prepared);

    assert!(super::subagent_queued::finalize_spawn_send_result(
        Err("Canal indisponible".to_string()),
        &parent.id,
        &child.id,
        "explorer",
    )
    .await
    .is_err());

    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    assert_eq!(subagent_hidden_reports::peek_reports(&parent.id).await.len(), 1);
    assert_eq!(
        session_store::get(&child.id)
            .await
            .expect("failed child")
            .subagent_status
            .as_deref(),
        Some(subagent_status::FAILED)
    );
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
