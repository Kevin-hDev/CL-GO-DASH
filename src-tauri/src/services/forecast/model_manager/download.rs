use crate::services::forecast::types::ModelDownloadProgress;
use futures_util::StreamExt;
use std::path::Component;
use std::path::Path;
use tauri::ipc::Channel;
use tokio::io::AsyncWriteExt;

const MAX_MODEL_FILES: usize = 128;

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
        download_single(
            &url,
            &dest,
            &mut downloaded,
            total_size,
            model_id,
            on_progress,
        )
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
    let url = format!("https://huggingface.co/api/models/{repo}/tree/main?recursive=true");
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

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parsing HF: {e}"))?;

    let tree = body.as_array().ok_or("Réponse HuggingFace invalide")?;

    let files: Vec<(String, u64)> = tree
        .iter()
        .filter_map(|s| {
            if s["type"].as_str()? != "file" {
                return None;
            }
            let name = s["path"].as_str()?;
            if !is_downloadable_model_file(name) {
                return None;
            }
            Some((name.to_string(), s["size"].as_u64().unwrap_or(0)))
        })
        .take(MAX_MODEL_FILES + 1)
        .collect();

    if files.is_empty() {
        return Err("Aucun fichier trouvé dans le repo".into());
    }
    if files.len() > MAX_MODEL_FILES {
        return Err("Repo modèle trop volumineux".into());
    }
    if files.iter().any(|(_, size)| *size == 0) {
        return Err("Taille modèle inconnue".into());
    }
    Ok(files)
}

fn is_downloadable_model_file(name: &str) -> bool {
    if name.is_empty() || name.len() > 240 || name == "README.md" || name.starts_with('.') {
        return false;
    }
    let path = Path::new(name);
    if path.is_absolute() {
        return false;
    }
    path.components()
        .all(|part| matches!(part, Component::Normal(_)))
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

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream: {e}"))?;
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

    tokio::fs::rename(&tmp, dest)
        .await
        .map_err(|e| format!("Rename: {e}"))
}
