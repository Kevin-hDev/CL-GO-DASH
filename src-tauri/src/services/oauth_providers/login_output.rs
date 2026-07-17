use super::{login_diagnostics::LoginDiagnostic, parse_login_hints, ProviderId};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio_util::sync::CancellationToken;

const MAX_LOGIN_OUTPUT: usize = 16 * 1024;
const MAX_ERROR_MESSAGE: usize = 2 * 1024;

#[derive(Default)]
pub struct LoginOutputSummary {
    pub account_access_required: bool,
}

pub async fn read<R: AsyncRead + Unpin>(
    mut output: R,
    app: tauri::AppHandle,
    provider: ProviderId,
    source: &'static str,
    diagnostic: LoginDiagnostic,
    account_error: CancellationToken,
) -> LoginOutputSummary {
    let mut collected = Vec::new();
    let mut chunk = [0u8; 1024];
    let mut last_hints = None;
    while collected.len() < MAX_LOGIN_OUTPUT {
        let read_len = (MAX_LOGIN_OUTPUT - collected.len()).min(chunk.len());
        let count = match output.read(&mut chunk[..read_len]).await {
            Ok(count) => count,
            Err(_) => {
                diagnostic.output(source, collected.len(), "read_error");
                return LoginOutputSummary::default();
            }
        };
        if count == 0 {
            let account_access_required = provider == ProviderId::Moonshot
                && kimi_account_access_required(&String::from_utf8_lossy(&collected));
            if account_access_required {
                account_error.cancel();
            }
            diagnostic.output(source, collected.len(), "eof");
            return LoginOutputSummary {
                account_access_required,
            };
        }
        collected.extend_from_slice(&chunk[..count]);
        let raw = String::from_utf8_lossy(&collected);
        if provider == ProviderId::Moonshot && kimi_account_access_required(&raw) {
            account_error.cancel();
            diagnostic.output(source, collected.len(), "account_error");
            return LoginOutputSummary {
                account_access_required: true,
            };
        }
        let hints = parse_login_hints(&raw);
        if hints != Default::default() && Some(&hints) != last_hints.as_ref() {
            super::login_progress::emit_verification(&app, provider, &hints, &diagnostic);
            last_hints = Some(hints);
        }
    }
    let account_access_required = provider == ProviderId::Moonshot
        && kimi_account_access_required(&String::from_utf8_lossy(&collected));
    if account_access_required {
        account_error.cancel();
    }
    diagnostic.output(source, collected.len(), "limit");
    LoginOutputSummary {
        account_access_required,
    }
}

pub async fn finish_readers(
    mut stdout: tokio::task::JoinHandle<LoginOutputSummary>,
    mut stderr: tokio::task::JoinHandle<LoginOutputSummary>,
    drain_timeout: std::time::Duration,
    diagnostic: &LoginDiagnostic,
) -> bool {
    let mut stdout_done = false;
    let mut stderr_done = false;
    let mut account_access_required = false;
    let deadline = tokio::time::sleep(drain_timeout);
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            result = &mut stdout, if !stdout_done => {
                stdout_done = true;
                account_access_required |= result
                    .map(|summary| summary.account_access_required)
                    .unwrap_or(false);
            },
            result = &mut stderr, if !stderr_done => {
                stderr_done = true;
                account_access_required |= result
                    .map(|summary| summary.account_access_required)
                    .unwrap_or(false);
            },
            _ = &mut deadline => break,
        }
        if account_access_required || (stdout_done && stderr_done) {
            break;
        }
    }
    if !stdout_done {
        stdout.abort();
    }
    if !stderr_done {
        stderr.abort();
    }
    diagnostic.output_drain(stdout_done, stderr_done, account_access_required);
    account_access_required
}

fn kimi_account_access_required(raw: &str) -> bool {
    raw.lines().any(|line| {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            return false;
        };
        if value.get("type").and_then(|item| item.as_str()) != Some("error") {
            return false;
        }
        let Some(message) = value.get("message").and_then(|item| item.as_str()) else {
            return false;
        };
        if message.len() > MAX_ERROR_MESSAGE {
            return false;
        }
        let message = message.to_ascii_lowercase();
        message.contains("no models available")
            || (message.contains("failed to get models")
                && (message.contains("402") || message.contains("payment required")))
    })
}

#[cfg(test)]
mod tests {
    use super::{finish_readers, kimi_account_access_required, LoginOutputSummary};

    #[test]
    fn detects_the_kimi_account_error_without_matching_unrelated_failures() {
        assert!(kimi_account_access_required(
            r#"{"type":"error","message":"Failed to get models: 402 Payment Required"}"#
        ));
        assert!(kimi_account_access_required(
            r#"{"type":"error","message":"No models available for the selected platform."}"#
        ));
        assert!(kimi_account_access_required(
            r#"{"type":"error","message":"Failed to get models: 402, message='Payment Required'"}"#
        ));
        assert!(!kimi_account_access_required(
            r#"{"type":"error","message":"Login failed: network unavailable"}"#
        ));
    }

    #[tokio::test]
    async fn output_drain_cannot_keep_a_finished_login_open_forever() {
        let stuck = tokio::spawn(std::future::pending::<LoginOutputSummary>());
        let finished = tokio::spawn(async { LoginOutputSummary::default() });
        let started = std::time::Instant::now();

        let account_error =
            finish_readers(
                stuck,
                finished,
                std::time::Duration::from_millis(20),
                &super::super::login_diagnostics::LoginDiagnostic::from_ui(
                    super::super::ProviderId::Moonshot,
                    &uuid::Uuid::new_v4().to_string(),
                )
                .unwrap(),
            )
            .await;

        assert!(!account_error);
        assert!(started.elapsed() < std::time::Duration::from_millis(200));
    }

    #[tokio::test]
    async fn account_error_survives_when_the_other_output_never_closes() {
        let detected = tokio::spawn(async {
            LoginOutputSummary {
                account_access_required: true,
            }
        });
        let stuck = tokio::spawn(std::future::pending::<LoginOutputSummary>());

        assert!(
            finish_readers(
                detected,
                stuck,
                std::time::Duration::from_millis(20),
                &super::super::login_diagnostics::LoginDiagnostic::from_ui(
                    super::super::ProviderId::Moonshot,
                    &uuid::Uuid::new_v4().to_string(),
                )
                .unwrap(),
            )
            .await
        );
    }
}
