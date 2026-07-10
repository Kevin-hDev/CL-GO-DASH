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
) -> bool {
    if super::subagent_registry::get_run_id_for_child(child_session_id)
        .await
        .is_none()
    {
        cleanup(child_session_id).await;
        return false;
    }
    let _ = super::subagent_completion::persist_terminal_completion(
        parent_session_id,
        child_session_id,
        subagent_type,
        super::subagent_status::FAILED,
        SUBAGENT_PANIC_SUMMARY,
    )
    .await;
    cleanup(child_session_id).await;
    true
}

async fn cleanup(child_session_id: &str) {
    super::subagent_working_dir::cleanup(child_session_id).await;
    super::session_store::remove_session_lock(child_session_id).await;
}
