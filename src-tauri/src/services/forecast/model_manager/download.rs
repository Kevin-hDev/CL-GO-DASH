use crate::services::model_downloads::{ModelDownloadPhase, ProgressUpdate};
use futures_util::StreamExt;
use std::path::Component;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

const MAX_MODEL_FILES: usize = 128;
const MIN_PROGRESS_STEP: f64 = 1.0;

pub async fn download_model(
    hf_repo: &str,
    hf_revision: Option<&str>,
    target_dir: &Path,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
) -> Result<(), String> {
    let files = list_hf_files(hf_repo, hf_revision).await?;
    let total_size: u64 = files.iter().map(|(_, size)| size).sum();
    let mut downloaded: u64 = 0;
    let mut last_percent_sent = 0.0;

    for (filename, _file_size) in &files {
        let revision = hf_revision.unwrap_or("main");
        let url = format!(
            "https://huggingface.co/{}/resolve/{}/{}",
            hf_repo, revision, filename
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
            cancel,
            on_progress,
            &mut last_percent_sent,
        )
        .await?;
    }

    on_progress(ProgressUpdate {
        phase: ModelDownloadPhase::Downloading,
        downloaded: total_size,
        total: total_size,
        percent: 99,
    });
    Ok(())
}

async fn list_hf_files(repo: &str, revision: Option<&str>) -> Result<Vec<(String, u64)>, String> {
    let rev = revision.unwrap_or("main");
    let url = format!("https://huggingface.co/api/models/{repo}/tree/{rev}?recursive=true");
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|_| "Erreur HuggingFace API".to_string())?;

    if !resp.status().is_success() {
        return Err("HuggingFace API erreur".into());
    }

    let body: serde_json::Value = resp.json().await.map_err(|_| "Parsing HF".to_string())?;

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
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
    last_percent_sent: &mut f64,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|_| "Download échoué".to_string())?;

    if !resp.status().is_success() {
        return Err("Download erreur".into());
    }

    let tmp = dest.with_extension("tmp");
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|_| "Création tmp échouée".to_string())?;

    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        if cancel.is_cancelled() {
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err("cancelled".into());
        }
        let chunk = chunk.map_err(|_| "Stream échoué".to_string())?;
        file.write_all(&chunk)
            .await
            .map_err(|_| "Écriture échouée".to_string())?;

        *downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
            (*downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        if percent >= *last_percent_sent + MIN_PROGRESS_STEP || percent >= 99.9 {
            *last_percent_sent = percent;
            on_progress(ProgressUpdate {
                phase: ModelDownloadPhase::Downloading,
                downloaded: *downloaded,
                total: total_size,
                percent: percent.round() as u8,
            });
        }
    }

    file.flush().await.map_err(|_| "Flush échoué".to_string())?;
    drop(file);

    tokio::fs::rename(&tmp, dest)
        .await
        .map_err(|_| "Rename échoué".to_string())
}
