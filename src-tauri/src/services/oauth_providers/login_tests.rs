use super::*;

#[tokio::test]
async fn account_error_interrupts_a_client_that_never_exits() {
    let cancel = CancellationToken::new();
    let account_error = CancellationToken::new();
    account_error.cancel();
    let pending = std::future::pending::<std::io::Result<std::process::ExitStatus>>();
    let started = std::time::Instant::now();

    let outcome = wait_for_stop(
        pending,
        &cancel,
        &account_error,
        std::time::Duration::from_secs(1),
    )
    .await;

    assert!(matches!(outcome, LoginStop::AccountError));
    assert!(started.elapsed() < std::time::Duration::from_millis(100));
}

#[tokio::test]
async fn process_cleanup_cannot_hold_the_login_slot_forever() {
    let started = std::time::Instant::now();
    let finished = super::super::login_wait::bounded_wait(
        std::future::pending::<()>(),
        std::time::Duration::from_millis(20),
    )
    .await;

    assert!(!finished);
    assert!(started.elapsed() < std::time::Duration::from_millis(200));
}
