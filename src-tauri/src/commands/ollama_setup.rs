use crate::services::ollama_lifecycle;
use serde::Serialize;
use tauri::ipc::Channel;

const FALLBACK_OLLAMA_VERSION: &str = "0.21.1";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaSetupProgress {
    pub completed: u64,
    pub total: u64,
    pub status: String,
}

#[tauri::command]
pub async fn is_ollama_installed() -> bool {
    ollama_lifecycle::is_ollama_ready()
}

#[tauri::command]
pub async fn download_ollama(on_progress: Channel<OllamaSetupProgress>) -> Result<(), String> {
    if ollama_lifecycle::ollama_binary_path().is_ok() {
        return Ok(());
    }
    let dest = ollama_lifecycle::ollama_bundle_dir();
    install_ollama_to(&dest, FALLBACK_OLLAMA_VERSION, &on_progress).await
}

#[tauri::command]
pub async fn update_ollama_binary(
    app: tauri::AppHandle,
    version: String,
    on_progress: Channel<OllamaSetupProgress>,
) -> Result<(), String> {
    let version = version.trim_start_matches('v');
    if !is_valid_semver(version) {
        return Err("version invalide".into());
    }

    let dest = ollama_lifecycle::ollama_bundle_dir();
    let staging = dest.with_file_name("ollama-bundle-staging");
    let _ = std::fs::remove_dir_all(&staging);

    if let Err(e) = install_ollama_to(&staging, version, &on_progress).await {
        let _ = std::fs::remove_dir_all(&staging);
        let _ = on_progress.send(OllamaSetupProgress {
            completed: 0, total: 0, status: "error".into(),
        });
        return Err(e);
    }

    ollama_lifecycle::stop_sidecar(&app);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let backup = dest.with_file_name("ollama-bundle-old");
    let _ = std::fs::remove_dir_all(&backup);
    if dest.exists() {
        std::fs::rename(&dest, &backup)
            .map_err(|e| { eprintln!("[ollama-update] backup: {e}"); "ollama-update-error".to_string() })?;
    }
    if let Err(e) = std::fs::rename(&staging, &dest) {
        let _ = std::fs::rename(&backup, &dest);
        eprintln!("[ollama-update] swap: {e}");
        return Err("ollama-update-error".into());
    }
    let _ = std::fs::remove_dir_all(&backup);

    let _ = on_progress.send(OllamaSetupProgress {
        completed: 0, total: 0, status: "restarting".into(),
    });

    ollama_lifecycle::start_sidecar(&app).map_err(|e| { eprintln!("[ollama-update] restart: {e}"); "ollama-restart-error".to_string() })?;

    eprintln!("[ollama-update] mis à jour vers v{version}");
    Ok(())
}

#[tauri::command]
pub async fn start_ollama_sidecar(app: tauri::AppHandle) -> Result<bool, String> {
    ollama_lifecycle::start_sidecar(&app)
}

#[tauri::command]
pub async fn restart_ollama_sidecar(app: tauri::AppHandle) -> Result<bool, String> {
    ollama_lifecycle::stop_sidecar(&app);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    ollama_lifecycle::start_sidecar(&app)
}

#[tauri::command]
pub async fn check_model_fits_vram(size_bytes: u64) -> bool {
    let vram_mb = crate::services::gpu_detect::detect_vram_mb().unwrap_or(0);
    if vram_mb == 0 { return true; }
    let model_mb = size_bytes / 1_048_576;
    model_mb < vram_mb
}

async fn install_ollama_to(
    dest: &std::path::Path,
    version: &str,
    on_progress: &Channel<OllamaSetupProgress>,
) -> Result<(), String> {
    let archives = archives_to_download();

    std::fs::create_dir_all(&dest).map_err(|e| {
        eprintln!("[ollama-setup] mkdir {}: {e}", dest.display());
        "Impossible de créer le dossier d'installation".to_string()
    })?;

    let checksums: Vec<Option<String>> = fetch_checksums(version, &archives).await;

    for (i, archive_name) in archives.iter().enumerate() {
        let url = format!(
            "https://github.com/ollama/ollama/releases/download/v{}/{}",
            version, archive_name
        );

        let status = if i == 0 { "downloading" } else { "downloading-rocm" };
        let _ = on_progress.send(OllamaSetupProgress {
            completed: 0, total: 0, status: status.into(),
        });

        let tmp = std::env::temp_dir().join(format!(
            "cl-go-ollama-{}-{archive_name}",
            std::process::id()
        ));
        if let Err(err) = download_file(&url, &tmp, on_progress).await {
            let _ = std::fs::remove_file(&tmp);
            let _ = std::fs::remove_dir_all(dest);
            return Err(err);
        }

        if let Some(Some(expected)) = checksums.get(i) {
            let _ = on_progress.send(OllamaSetupProgress {
                completed: 0, total: 0, status: "verifying".into(),
            });
            if let Err(err) = super::ollama_checksum::verify_file_sha256(&tmp, expected) {
                let _ = std::fs::remove_file(&tmp);
                let _ = std::fs::remove_dir_all(dest);
                return Err(err);
            }
        }

        let _ = on_progress.send(OllamaSetupProgress {
            completed: 0, total: 0, status: "extracting".into(),
        });

        if let Err(err) = super::ollama_extract::extract_overlay(&tmp, dest, archive_name) {
            let _ = std::fs::remove_dir_all(dest);
            let _ = std::fs::remove_file(&tmp);
            return Err(err);
        }
        let _ = std::fs::remove_file(&tmp);
    }

    let binary = find_binary_in(dest).ok_or_else(|| {
        let _ = std::fs::remove_dir_all(dest);
        "installation incomplète: binaire Ollama introuvable".to_string()
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755));
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("xattr")
            .args(["-d", "com.apple.quarantine"])
            .arg(&binary)
            .output();
        eprintln!("[ollama] quarantine attribute supprimé");
    }

    write_version_file(dest, version);
    eprintln!("[ollama-setup] installé v{version}: {}", binary.display());
    Ok(())
}

use super::ollama_bundle_utils::{
    archives_to_download, find_binary_in, is_valid_semver, write_version_file,
};
use super::ollama_download::download_file;

async fn fetch_checksums(version: &str, archives: &[&str]) -> Vec<Option<String>> {
    let mut result = Vec::with_capacity(archives.len());
    for name in archives {
        match super::ollama_checksum::fetch_expected_hash(version, name).await {
            Ok(hash) => result.push(Some(hash)),
            Err(e) => {
                eprintln!("[ollama-setup] checksum unavailable for {name}: {e}");
                result.push(None);
            }
        }
    }
    result
}

