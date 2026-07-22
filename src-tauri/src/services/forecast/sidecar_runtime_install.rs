use super::manifest::expected_requirements;
use super::paths::{
    commit_staged_runtime, remove_if_exists, runtime_paths, runtime_ready_at, write_stamp,
};
use crate::services::process_tree;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeInstallStep {
    CreatingEnvironment,
    PreparingInstaller,
    InstallingDependencies,
    Finalizing,
}

pub(super) fn prepare_runtime(
    sidecar_dir: &Path,
    family_id: &str,
    cancel: &CancellationToken,
    on_progress: &dyn Fn(RuntimeInstallStep),
) -> Result<PathBuf, String> {
    let requirements = expected_requirements(sidecar_dir, family_id)?;
    let paths = runtime_paths(sidecar_dir, family_id)?;
    if runtime_ready_at(&paths.live, &requirements) {
        remove_if_exists(&paths.staging)?;
        remove_if_exists(&paths.backup)?;
        return Ok(paths.python_in(&paths.live));
    }
    remove_if_exists(&paths.staging)?;
    let result = (|| {
        check_cancel(cancel)?;
        on_progress(RuntimeInstallStep::CreatingEnvironment);
        let python = find_python()?;
        run_cancellable(
            Command::new(python)
                .args(["-m", "venv"])
                .arg(&paths.staging),
            cancel,
            "Création du runtime Forecast impossible",
        )?;
        let staged_python = paths.python_in(&paths.staging);
        on_progress(RuntimeInstallStep::PreparingInstaller);
        run_cancellable(
            Command::new(&staged_python).args([
                "-m",
                "pip",
                "install",
                "--no-input",
                "--no-cache-dir",
                "--upgrade",
                "pip",
            ]),
            cancel,
            "Initialisation du runtime Forecast impossible",
        )?;
        let manifest = paths.staging.join("requirements.txt");
        std::fs::write(&manifest, &requirements)
            .map_err(|_| "Préparation du runtime Forecast impossible".to_string())?;
        on_progress(RuntimeInstallStep::InstallingDependencies);
        run_cancellable(
            Command::new(&staged_python)
                .args(["-m", "pip", "install", "--no-input", "--no-cache-dir", "-r"])
                .arg(&manifest),
            cancel,
            "Installation du moteur Forecast impossible",
        )?;
        check_cancel(cancel)?;
        write_stamp(&paths.staging, &requirements)?;
        if !runtime_ready_at(&paths.staging, &requirements) {
            return Err("Validation du runtime Forecast impossible".to_string());
        }
        on_progress(RuntimeInstallStep::Finalizing);
        check_cancel(cancel)?;
        commit_staged_runtime(&paths)?;
        Ok(paths.python_in(&paths.live))
    })();
    if result.is_err() {
        let _ = remove_if_exists(&paths.staging);
    }
    result
}

fn check_cancel(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("cancelled".to_string())
    } else {
        Ok(())
    }
}

fn find_python() -> Result<PathBuf, String> {
    for candidate in [
        "python3.12",
        "python3.13",
        "python3.14",
        "python3",
        "python",
    ] {
        if let Ok(path) = which::which(candidate) {
            return Ok(path);
        }
    }
    Err("Runtime Python introuvable".to_string())
}

fn run_cancellable(
    command: &mut Command,
    cancel: &CancellationToken,
    message: &str,
) -> Result<(), String> {
    command
        .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
        .env("PYTHONUNBUFFERED", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command.spawn().map_err(|_| message.to_string())?;
    loop {
        if cancel.is_cancelled() {
            process_tree::kill(child.id(), process_tree::ProcessKind::ForecastRuntime);
            let _ = child.wait();
            return Err("cancelled".to_string());
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(_)) | Err(_) => return Err(message.to_string()),
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}
