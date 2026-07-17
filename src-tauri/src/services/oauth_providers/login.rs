use super::{
    command_spec, login_diagnostics::LoginDiagnostic, process_environment, profile_dir,
    ProcessKind, ProviderId,
};
use super::login_wait::{bounded_wait, wait_for_stop, LoginStop};
use tauri::Emitter;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

const LOGIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10 * 60);
const OUTPUT_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(500);
const PROCESS_STOP_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(500);
const STATUS_EVENT: &str = "oauth-provider-status-changed";

enum LoginFailure {
    AccountAccessRequired,
    Generic,
}

pub async fn run(
    app: tauri::AppHandle,
    provider: ProviderId,
    diagnostic: LoginDiagnostic,
) -> Result<(), String> {
    if !matches!(provider, ProviderId::Moonshot | ProviderId::Xai) {
        return Err("Provider OAuth invalide".to_string());
    }
    let cancel = match super::login_registry::register(provider).await {
        Ok(cancel) => {
            diagnostic.stage("login_slot_acquired");
            cancel
        }
        Err(error) => {
            diagnostic.stage("login_slot_rejected");
            return Err(error);
        }
    };
    let result = run_registered(&app, provider, cancel.clone(), &diagnostic).await;
    super::login_registry::release(provider).await;
    diagnostic.stage("login_slot_released");
    match &result {
        Ok(()) => {
            super::status::mark_connected(provider)?;
            diagnostic.current_state("login_succeeded");
            super::login_progress::emit(&app, provider, "success", &diagnostic);
            let _ = app.emit(STATUS_EVENT, ());
        }
        Err(LoginFailure::AccountAccessRequired) if !cancel.is_cancelled() => {
            diagnostic.current_state("account_access_required");
            super::login_progress::emit(
                &app,
                provider,
                "account_access_required",
                &diagnostic,
            );
        }
        Err(LoginFailure::Generic) if !cancel.is_cancelled() => {
            diagnostic.current_state("login_failed");
            super::login_progress::emit(&app, provider, "error", &diagnostic);
        }
        Err(_) => diagnostic.stage("login_cancelled"),
    }
    result.map_err(|_| "Connexion impossible".to_string())
}

async fn run_registered(
    app: &tauri::AppHandle,
    provider: ProviderId,
    cancel: CancellationToken,
    diagnostic: &LoginDiagnostic,
) -> Result<(), LoginFailure> {
    let spec = command_spec(provider, ProcessKind::Login);
    diagnostic.current_state("preflight");
    let binary = super::status::binary_path(provider).ok_or_else(|| {
        diagnostic.stage("binary_missing");
        LoginFailure::Generic
    })?;
    diagnostic.stage("binary_ready");
    let home = profile_dir(provider);
    tokio::fs::create_dir_all(&home)
        .await
        .map_err(|_| LoginFailure::Generic)?;
    tokio::fs::create_dir_all(home.join("agent-home"))
        .await
        .map_err(|_| LoginFailure::Generic)?;
    super::logout::remove_credentials_in(&home, provider)
        .await
        .map_err(|_| LoginFailure::Generic)?;
    diagnostic.current_state("credentials_cleared");
    super::status::mark_connected(provider).map_err(|_| LoginFailure::Generic)?;
    let mut command = Command::new(binary);
    for (name, value) in process_environment(provider) {
        command.env(name, value);
    }
    let mut child = command
        .args(spec.args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| LoginFailure::Generic)?;
    diagnostic.stage("process_spawned");
    super::login_progress::emit(app, provider, "waiting", diagnostic);
    let stdout = child.stdout.take().ok_or(LoginFailure::Generic)?;
    let stderr = child.stderr.take().ok_or(LoginFailure::Generic)?;
    let account_error = CancellationToken::new();
    let stdout_task = tokio::spawn(super::login_output::read(
        stdout,
        app.clone(),
        provider,
        "stdout",
        diagnostic.clone(),
        account_error.clone(),
    ));
    let stderr_task = tokio::spawn(super::login_output::read(
        stderr,
        app.clone(),
        provider,
        "stderr",
        diagnostic.clone(),
        account_error.clone(),
    ));
    let status = match wait_for_stop(child.wait(), &cancel, &account_error, LOGIN_TIMEOUT).await {
        LoginStop::Process(result) => result.map_err(|_| {
            diagnostic.stage("process_wait_failed");
            LoginFailure::Generic
        })?,
        LoginStop::AccountError => {
            stdout_task.abort();
            stderr_task.abort();
            stop_process(&mut child, diagnostic).await;
            diagnostic.stage("account_error_process_stopped");
            return Err(LoginFailure::AccountAccessRequired);
        }
        LoginStop::Cancelled => {
            stdout_task.abort();
            stderr_task.abort();
            stop_process(&mut child, diagnostic).await;
            super::login_progress::emit(app, provider, "cancelled", diagnostic);
            diagnostic.stage("process_cancelled");
            return Err(LoginFailure::Generic);
        }
        LoginStop::Timeout => {
            stdout_task.abort();
            stderr_task.abort();
            stop_process(&mut child, diagnostic).await;
            diagnostic.stage("process_timeout");
            return Err(LoginFailure::Generic);
        }
    };
    diagnostic.process_exit(status.success(), status.code());
    let account_access_required =
        super::login_output::finish_readers(
            stdout_task,
            stderr_task,
            OUTPUT_DRAIN_TIMEOUT,
            diagnostic,
        )
        .await;
    if account_access_required {
        return Err(LoginFailure::AccountAccessRequired);
    }
    diagnostic.current_state("post_process");
    if status.success() && super::status::credentials_present_in(&home, provider) {
        Ok(())
    } else {
        Err(LoginFailure::Generic)
    }
}

async fn stop_process(child: &mut tokio::process::Child, diagnostic: &LoginDiagnostic) {
    diagnostic.stage("process_stop_started");
    if child.start_kill().is_err() {
        diagnostic.stage("process_stop_signal_failed");
        return;
    }
    let stopped = bounded_wait(child.wait(), PROCESS_STOP_TIMEOUT).await;
    diagnostic.stage(if stopped {
        "process_stop_finished"
    } else {
        "process_stop_deadline"
    });
}

#[cfg(test)]
#[path = "login_tests.rs"]
mod tests;
