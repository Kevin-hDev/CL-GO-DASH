use std::future::Future;

pub const SUBAGENT_PANIC_SUMMARY: &str = "Le sous-agent n'a pas pu terminer correctement.";

pub async fn run_guarded<F, Recover, RecoverFuture>(future: F, recover: Recover)
where
    F: Future<Output = ()> + Send + 'static,
    Recover: FnOnce() -> RecoverFuture,
    RecoverFuture: Future<Output = ()>,
{
    if tokio::spawn(future).await.is_err() {
        recover().await;
    }
}

pub async fn recover_panicked_completion(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    run_id: &str,
    execution_id: &str,
    expected_worktree_path: Option<&str>,
    emitter: Option<&super::stream_events::AgentEventEmitter>,
) -> bool {
    let completion = super::subagent_completion_events::persist_terminal(
        parent_session_id,
        child_session_id,
        subagent_type,
        super::subagent_status::FAILED,
        SUBAGENT_PANIC_SUMMARY,
        run_id,
        execution_id,
        false,
        emitter,
    )
    .await;
    cleanup(child_session_id, execution_id, expected_worktree_path).await;
    !matches!(completion, Ok(None))
}

async fn cleanup(
    child_session_id: &str,
    execution_id: &str,
    expected_worktree_path: Option<&str>,
) {
    super::subagent_working_dir::cleanup_owned(
        child_session_id,
        execution_id,
        expected_worktree_path,
    )
    .await;
    super::session_store::remove_session_lock(child_session_id).await;
}
