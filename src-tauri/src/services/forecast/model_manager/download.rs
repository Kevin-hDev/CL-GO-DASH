use crate::services::forecast::types::ModelDownloadProgress;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::path::Path;
use tauri::ipc::Channel;
use tokio::io::AsyncWriteExt;

pub async fn download_model(
    hf_repo: &str,
    target_dir: &Path,
    model_id: &str,
    on_progress: &Channel<ModelDownloadProgress>,
) -> Result<(), String> {
    let files = list_hf_files(hf_repo).await?;
    let total_size: u64 = files.iter().map(|(_, size)| size).sum();
    let mut downloaded: u64 = 0;

    for (filename, _file_size) in &files {
        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            hf_repo, filename
        );
        let dest = target_dir.join(filename);
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        download_single(&url, &dest, &mut downloaded, total_size, model_id, on_progress)
            .await?;
    }

    let _ = on_progress.send(ModelDownloadProgress {
        model_name: model_id.to_string(),
        downloaded: total_size,
        total: total_size,
        percent: 100.0,
    });
    Ok(())
}

async fn list_hf_files(repo: &str) -> Result<Vec<(String, u64)>, String> {
    let url = format!("https://huggingface.co/api/models/{repo}");
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Erreur HuggingFace API: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HuggingFace API erreur: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Parsing HF: {e}"))?;

    let siblings = body["siblings"]
        .as_array()
        .ok_or("Champ siblings manquant")?;

    let files: Vec<(String, u64)> = siblings
        .iter()
        .filter_map(|s| {
            let name = s["rfilename"].as_str()?;
            if name.starts_with('.') || name == "README.md" {
                return None;
            }
            let size = s["size"].as_u64().unwrap_or(0);
            Some((name.to_string(), size))
        })
        .collect();

    if files.is_empty() {
        return Err("Aucun fichier trouvé dans le repo".into());
    }
    Ok(files)
}

async fn download_single(
    url: &str,
    dest: &Path,
    downloaded: &mut u64,
    total_size: u64,
    model_id: &str,
    on_progress: &Channel<ModelDownloadProgress>,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download échoué: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Download erreur: {}", resp.status()));
    }

    let tmp = dest.with_extension("tmp");
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| format!("Création tmp: {e}"))?;

    let mut stream = resp.bytes_stream();
    let mut hasher = Sha256::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream: {e}"))?;
        hasher.update(&chunk);
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Écriture: {e}"))?;

        *downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
            (*downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        let _ = on_progress.send(ModelDownloadProgress {
            model_name: model_id.to_string(),
            downloaded: *downloaded,
            total: total_size,
            percent,
        });
    }

    file.flush().await.map_err(|e| format!("Flush: {e}"))?;
    drop(file);

    let _hash = format!("{:x}", hasher.finalize());

    tokio::fs::rename(&tmp, dest)
        .await
        .map_err(|e| format!("Rename: {e}"))
}
