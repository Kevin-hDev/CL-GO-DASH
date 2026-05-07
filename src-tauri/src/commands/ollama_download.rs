use tauri::ipc::Channel;
use super::ollama_setup::OllamaSetupProgress;

const MIN_ARCHIVE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_BINARY_BYTES: u64 = 3 * 1024 * 1024 * 1024;
const DOWNLOAD_TIMEOUT_SECS: u64 = 1800;

pub async fn download_file(
    url: &str,
    dest: &std::path::Path,
    on_progress: &Channel<OllamaSetupProgress>,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .build()
        .map_err(|e| { eprintln!("[ollama-dl] client: {e}"); "ollama-download-error".to_string() })?;

    let resp = client
        .get(url)
        .header("User-Agent", "CL-GO-DASH")
        .send()
        .await
        .map_err(|e| { eprintln!("[ollama-dl] network: {e}"); "ollama-download-error".to_string() })?;

    if !resp.status().is_success() {
        return Err("ollama-download-refused".into());
    }

    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    let total = resp.content_length().unwrap_or(0);
    if content_type.contains("text/html") {
        return Err("ollama-download-invalid".into());
    }
    if total > 0 && total < MIN_ARCHIVE_BYTES {
        return Err("ollama-download-too-small".into());
    }
    if total > MAX_BINARY_BYTES {
        return Err("ollama-download-too-large".into());
    }

    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| { eprintln!("[ollama-dl] fs create: {e}"); "ollama-write-error".to_string() })?;

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| { eprintln!("[ollama-dl] stream: {e}"); "ollama-download-error".to_string() })?;
        downloaded += chunk.len() as u64;
        if downloaded > MAX_BINARY_BYTES {
            let _ = tokio::fs::remove_file(dest).await;
            return Err("ollama-download-too-large".into());
        }
        file.write_all(&chunk)
            .await
            .map_err(|e| { eprintln!("[ollama-dl] write: {e}"); "ollama-write-error".to_string() })?;
        let _ = on_progress.send(OllamaSetupProgress {
            completed: downloaded,
            total,
            status: "downloading".into(),
        });
    }

    file.flush().await.map_err(|e| { eprintln!("[ollama-dl] flush: {e}"); "ollama-write-error".to_string() })?;

    let size = tokio::fs::metadata(dest)
        .await
        .map_err(|e| { eprintln!("[ollama-dl] metadata: {e}"); "ollama-write-error".to_string() })?
        .len();
    if size < MIN_ARCHIVE_BYTES {
        let _ = tokio::fs::remove_file(dest).await;
        return Err("ollama-download-incomplete".into());
    }

    Ok(())
}
