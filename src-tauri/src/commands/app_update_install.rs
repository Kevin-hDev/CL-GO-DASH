use serde::Serialize;
use tauri::ipc::Channel;

use super::app_update_assets::{current_platform, temp_extension};
use super::app_update_install_scripts::spawn_update_script;
use super::app_update_install_temp::create_unique_temp_file;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub completed: u64,
    pub total: u64,
}

const TRUSTED_UPDATE_PREFIX: &str = "https://github.com/Kevin-hDev/CL-GO-DASH/releases/download/";

fn validate_update_url(url: &str) -> Result<(), String> {
    if !url.starts_with(TRUSTED_UPDATE_PREFIX) {
        return Err("update-url-untrusted".to_string());
    }
    if url.contains("..") || url.contains('\0') {
        return Err("update-url-invalid".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn download_app_update(
    app: tauri::AppHandle,
    asset_url: String,
    on_progress: Channel<DownloadProgress>,
) -> Result<(), String> {
    validate_update_url(&asset_url)?;

    let client = reqwest::Client::new();
    let resp = client
        .get(&asset_url)
        .header("User-Agent", "CL-GO-DASH")
        .send()
        .await
        .map_err(|e| {
            eprintln!("[update] network: {e}");
            "update-download-error".to_string()
        })?;

    if !resp.status().is_success() {
        return Err("download failed".into());
    }

    let total = resp.content_length().unwrap_or(0);
    let ext = temp_extension(current_platform());
    let (tmp, file) = create_unique_temp_file("CL-GO-update", &format!(".{ext}"))?;
    let mut file = tokio::fs::File::from_std(file);

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            eprintln!("[update] stream: {e}");
            "update-download-error".to_string()
        })?;
        file.write_all(&chunk).await.map_err(|e| {
            eprintln!("[update] write: {e}");
            "update-write-error".to_string()
        })?;
        downloaded += chunk.len() as u64;
        let _ = on_progress.send(DownloadProgress {
            completed: downloaded,
            total,
        });
    }

    file.flush().await.map_err(|e| {
        eprintln!("[update] flush: {e}");
        "update-write-error".to_string()
    })?;
    drop(file);

    spawn_update_script(&tmp)?;
    app.exit(0);

    Ok(())
}
