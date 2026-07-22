use crate::services::process_tree;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

const SMOKE_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const SMOKE_MARKER: &str = ".smoke-v1";
const MAX_SOURCE_FILES: usize = 128;
const MAX_SOURCE_BYTES: u64 = 1024 * 1024;

pub(super) async fn validate_model(
    runtime_python: &Path,
    sidecar: &Path,
    models: &Path,
    model_dir: &Path,
    family_id: &str,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let python = runtime_python.to_path_buf();
    let script = sidecar.join("test_model_smoke.py");
    let models = models.to_path_buf();
    let model_name = model_name(model_dir)?.to_string();
    let family = family_id.to_string();
    let cancellation = cancel.clone();
    tokio::task::spawn_blocking(move || {
        run_smoke(
            &python,
            &script,
            &models,
            &model_name,
            &family,
            &cancellation,
        )
    })
    .await
    .map_err(|_| "Validation du modèle Forecast impossible".to_string())??;
    let fingerprint = source_fingerprint(sidecar)?;
    tokio::fs::write(model_dir.join(SMOKE_MARKER), fingerprint)
        .await
        .map_err(|_| "Validation du modèle Forecast impossible".to_string())
}

pub(super) fn is_validated(model_dir: &Path, sidecar: &Path) -> bool {
    let Ok(expected) = source_fingerprint(sidecar) else {
        return false;
    };
    std::fs::read(model_dir.join(SMOKE_MARKER)).is_ok_and(|value| value == expected.as_bytes())
}

fn source_fingerprint(sidecar: &Path) -> Result<String, String> {
    let runtime = sidecar.join("forecast_runtime");
    let mut files = std::fs::read_dir(runtime)
        .map_err(|_| "Validation du modèle Forecast impossible".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("py"))
        .collect::<Vec<_>>();
    files.sort();
    if files.is_empty() || files.len() > MAX_SOURCE_FILES {
        return Err("Validation du modèle Forecast impossible".into());
    }
    let mut digest = Sha256::new();
    for path in files {
        let metadata = std::fs::metadata(&path)
            .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
        if !metadata.is_file() || metadata.len() > MAX_SOURCE_BYTES {
            return Err("Validation du modèle Forecast impossible".into());
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or("Validation du modèle Forecast impossible")?;
        let body = std::fs::read(&path)
            .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
        digest.update((name.len() as u64).to_be_bytes());
        digest.update(name.as_bytes());
        digest.update((body.len() as u64).to_be_bytes());
        digest.update(body);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn run_smoke(
    python: &Path,
    script: &Path,
    models: &Path,
    model_name: &str,
    family_id: &str,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if !script.is_file() || !python.is_file() {
        return Err("Validation du modèle Forecast impossible".into());
    }
    let mut child = Command::new(python)
        .arg(script)
        .current_dir(script.parent().unwrap_or(sidecar_fallback()))
        .env("FORECAST_SMOKE", "1")
        .env("FORECAST_SMOKE_FAMILY", family_id)
        .env("FORECAST_SMOKE_MODEL", model_name)
        .env("FORECAST_SMOKE_MODELS_DIR", models)
        .env("FORECAST_SMOKE_DEVICE", "cpu")
        .env("HF_HUB_OFFLINE", "1")
        .env("TRANSFORMERS_OFFLINE", "1")
        .env("TABPFN_DISABLE_TELEMETRY", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
    let started = Instant::now();
    loop {
        if cancel.is_cancelled() || started.elapsed() >= SMOKE_TIMEOUT {
            process_tree::kill(child.id(), process_tree::ProcessKind::ForecastRuntime);
            let _ = child.wait();
            return Err(if cancel.is_cancelled() {
                "cancelled".into()
            } else {
                "Validation du modèle Forecast expirée".into()
            });
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(_)) | Err(_) => return Err("Validation du modèle Forecast impossible".into()),
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

fn model_name(model_dir: &Path) -> Result<&str, String> {
    model_dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && !name.contains(['/', '\\']))
        .ok_or_else(|| "Validation du modèle Forecast impossible".to_string())
}

fn sidecar_fallback() -> &'static Path {
    Path::new(".")
}

#[cfg(test)]
mod tests {
    use super::{model_name, source_fingerprint};
    use std::path::Path;

    #[test]
    fn model_name_only_uses_the_final_path_component() {
        assert_eq!(
            model_name(Path::new("/models/.safe.staging")).unwrap(),
            ".safe.staging"
        );
        assert!(model_name(Path::new("/")).is_err());
    }

    #[test]
    fn smoke_fingerprint_changes_with_adapter_source() {
        let temp = tempfile::tempdir().unwrap();
        let runtime = temp.path().join("forecast_runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(runtime.join("adapter.py"), "first").unwrap();
        let first = source_fingerprint(temp.path()).unwrap();
        std::fs::write(runtime.join("adapter.py"), "second").unwrap();

        assert_ne!(first, source_fingerprint(temp.path()).unwrap());
    }
}
