use crate::services::ollama_lifecycle;
use serde::Serialize;
use tauri::ipc::Channel;

const OLLAMA_VERSION: &str = "0.21.1";
const MIN_ARCHIVE_BYTES: u64 = 10 * 1024 * 1024;

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
    std::fs::create_dir_all(&dest).map_err(|e| format!("mkdir: {e}"))?;

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

async fn download_file(
    url: &str,
    dest: &std::path::Path,
    on_progress: &Channel<OllamaSetupProgress>,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "CL-GO-DASH")
        .send()
        .await
        .map_err(|e| format!("network: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("téléchargement refusé (HTTP {})", resp.status()));
    }

    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    let total = resp.content_length().unwrap_or(0);
    if content_type.contains("text/html") {
        return Err("réponse invalide: page HTML reçue au lieu de l’archive Ollama".into());
    }
    if total > 0 && total < MIN_ARCHIVE_BYTES {
        return Err(format!(
            "archive Ollama invalide: taille trop petite ({} octets)",
            total
        ));
    }

    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("fs: {e}"))?;

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("write: {e}"))?;
        downloaded += chunk.len() as u64;
        let _ = on_progress.send(OllamaSetupProgress {
            completed: downloaded,
            total,
            status: "downloading".into(),
        });
    }

    file.flush().await.map_err(|e| format!("flush: {e}"))?;

    let size = tokio::fs::metadata(dest)
        .await
        .map_err(|e| format!("metadata: {e}"))?
        .len();
    if size < MIN_ARCHIVE_BYTES {
        let _ = tokio::fs::remove_file(dest).await;
        return Err(format!(
            "archive Ollama invalide: fichier incomplet ({} octets)",
            size
        ));
    }

    Ok(())
}
