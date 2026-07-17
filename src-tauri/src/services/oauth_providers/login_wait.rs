use tokio_util::sync::CancellationToken;

pub enum LoginStop {
    Process(std::io::Result<std::process::ExitStatus>),
    AccountError,
    Cancelled,
    Timeout,
}

pub async fn wait_for_stop<F>(
    process: F,
    cancel: &CancellationToken,
    account_error: &CancellationToken,
    timeout: std::time::Duration,
) -> LoginStop
where
    F: std::future::Future<Output = std::io::Result<std::process::ExitStatus>>,
{
    tokio::select! {
        result = process => LoginStop::Process(result),
        _ = account_error.cancelled() => LoginStop::AccountError,
        _ = cancel.cancelled() => LoginStop::Cancelled,
        _ = tokio::time::sleep(timeout) => LoginStop::Timeout,
    }
}

pub async fn bounded_wait<F, T>(future: F, timeout: std::time::Duration) -> bool
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(timeout, future).await.is_ok()
}
