use crate::services::ollama_lifecycle;
use serde::Serialize;
use tauri::ipc::Channel;

const OLLAMA_VERSION: &str = "0.21.1";

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
    let dest = ollama_lifecycle::ollama_bundle_dir();
    let binary_name = if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    };
    if dest.join(binary_name).exists() {
        return Ok(());
    }

    let archive_name = select_archive_name();

    let url = format!(
        "https://github.com/ollama/ollama/releases/download/v{}/{}",
        OLLAMA_VERSION, archive_name
    );

    let _ = on_progress.send(OllamaSetupProgress {
        completed: 0,
        total: 0,
        status: "downloading".into(),
    });

    let tmp = std::env::temp_dir().join(format!("cl-go-ollama-{}", archive_name));
    if let Err(err) = download_file(&url, &tmp, &on_progress).await {
        let _ = std::fs::remove_file(&tmp);
        return Err(err);
    }

    let _ = on_progress.send(OllamaSetupProgress {
        completed: 0,
        total: 0,
        status: "extracting".into(),
    });

    let _ = std::fs::remove_dir_all(&dest);
    std::fs::create_dir_all(&dest).map_err(|e| {
        eprintln!("[ollama-setup] mkdir {}: {e}", dest.display());
        "Impossible de créer le dossier d'installation".to_string()
    })?;

    if let Err(err) = super::ollama_extract::extract_archive(&tmp, &dest, archive_name, binary_name)
    {
        let _ = std::fs::remove_dir_all(&dest);
        let _ = std::fs::remove_file(&tmp);
        return Err(err);
    }
    let _ = std::fs::remove_file(&tmp);

    #[cfg(unix)]
    {
        let bin = dest.join("ollama");
        if bin.exists() {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755));
        }
    }

    #[cfg(target_os = "macos")]
    {
        let bin = dest.join("ollama");
        if bin.exists() {
            let _ = std::process::Command::new("xattr")
                .args(["-d", "com.apple.quarantine"])
                .arg(&bin)
                .output();
            eprintln!("[ollama] quarantine attribute supprimé");
        }
    }

    if !dest.join(binary_name).is_file() {
        let _ = std::fs::remove_dir_all(&dest);
        return Err("installation incomplète: binaire Ollama introuvable".into());
    }

    Ok(())
}

#[tauri::command]
pub async fn start_ollama_sidecar(app: tauri::AppHandle) -> Result<bool, String> {
    crate::services::ollama_lifecycle::start_sidecar(&app)
}

#[tauri::command]
pub async fn restart_ollama_sidecar(app: tauri::AppHandle) -> Result<bool, String> {
    crate::services::ollama_lifecycle::stop_sidecar(&app);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    crate::services::ollama_lifecycle::start_sidecar(&app)
}

#[tauri::command]
pub async fn check_model_fits_vram(size_bytes: u64) -> bool {
    let vram_mb = crate::services::gpu_detect::detect_vram_mb().unwrap_or(0);
    if vram_mb == 0 { return true; }
    let model_mb = size_bytes / 1_048_576;
    model_mb < vram_mb
}

fn select_archive_name() -> &'static str {
    if cfg!(target_os = "macos") {
        return "ollama-darwin.tgz";
    }

    if cfg!(target_os = "windows") {
        // Sous Windows, le zip ROCm est un complément au bundle principal,
        // pas un remplaçant autonome du CLI.
        return "ollama-windows-amd64.zip";
    }

    use crate::services::gpu_detect::{self, GpuVendor};
    let gpu = gpu_detect::detect();
    eprintln!("[ollama] GPU détecté : {:?}", gpu);

    match gpu {
        GpuVendor::Amd => "ollama-linux-amd64-rocm.tar.zst",
        _ => "ollama-linux-amd64.tar.zst",
    }
}

use super::ollama_download::download_file;
