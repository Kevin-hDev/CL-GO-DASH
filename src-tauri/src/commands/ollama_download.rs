use tauri::ipc::Channel;
use super::ollama_setup::OllamaSetupProgress;

const MIN_ARCHIVE_BYTES: u64 = 10 * 1024 * 1024;

pub async fn download_file(
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
        .map_err(|e| { eprintln!("[ollama-dl] network: {e}"); "Erreur réseau lors du téléchargement".to_string() })?;

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
        return Err("réponse invalide: page HTML reçue au lieu de l'archive Ollama".into());
    }
    if total > 0 && total < MIN_ARCHIVE_BYTES {
        return Err(format!(
            "archive Ollama invalide: taille trop petite ({} octets)",
            total
        ));
    }

    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| { eprintln!("[ollama-dl] fs create: {e}"); "Erreur d'écriture sur le disque".to_string() })?;

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| { eprintln!("[ollama-dl] stream: {e}"); "Erreur pendant le téléchargement".to_string() })?;
        file.write_all(&chunk)
            .await
            .map_err(|e| { eprintln!("[ollama-dl] write: {e}"); "Erreur d'écriture sur le disque".to_string() })?;
        downloaded += chunk.len() as u64;
        let _ = on_progress.send(OllamaSetupProgress {
            completed: downloaded,
            total,
            status: "downloading".into(),
        });
    }

    file.flush().await.map_err(|e| { eprintln!("[ollama-dl] flush: {e}"); "Erreur d'écriture sur le disque".to_string() })?;

    let size = tokio::fs::metadata(dest)
        .await
        .map_err(|e| { eprintln!("[ollama-dl] metadata: {e}"); "Erreur de vérification du fichier".to_string() })?
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
