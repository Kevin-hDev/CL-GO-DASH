use crate::services::model_downloads::{ModelDownloadPhase, ProgressUpdate};
use std::path::Component;
use std::path::Path;
use tokio_util::sync::CancellationToken;

#[path = "download_io.rs"]
mod download_io;

const MAX_MODEL_FILES: usize = 128;
const MAX_HF_TREE_BYTES: usize = 2 * 1024 * 1024;
const MAX_MODEL_BYTES: u64 = 32 * 1024 * 1024 * 1024;
const MIN_PROGRESS_STEP: f64 = 1.0;

pub async fn download_model(
    hf_repo: &str,
    hf_revision: Option<&str>,
    selected_file: Option<&str>,
    target_dir: &Path,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
) -> Result<(), String> {
    let files = list_hf_files(hf_repo, hf_revision, selected_file).await?;
    let total_size = checked_total_size(&files)?;
    let mut downloaded: u64 = 0;
    let mut last_percent_sent = 0.0;

    for (filename, file_size) in &files {
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
            *file_size,
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

async fn list_hf_files(
    repo: &str,
    revision: Option<&str>,
    selected_file: Option<&str>,
) -> Result<Vec<(String, u64)>, String> {
    let rev = revision.unwrap_or("main");
    let url = format!("https://huggingface.co/api/models/{repo}/tree/{rev}?recursive=true");
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| "Erreur HuggingFace API".to_string())?;
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|_| "Erreur HuggingFace API".to_string())?;

    if !resp.status().is_success() {
        return Err("HuggingFace API erreur".into());
    }

    let body = download_io::read_bounded_json(resp, MAX_HF_TREE_BYTES).await?;

    let tree = body.as_array().ok_or("Réponse HuggingFace invalide")?;

    let files: Vec<(String, u64)> = tree
        .iter()
        .filter_map(|s| {
            if s["type"].as_str()? != "file" {
                return None;
            }
            let name = s["path"].as_str()?;
            if !is_downloadable_model_file(name)
                || selected_file.is_some_and(|selected| selected != name)
            {
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

fn checked_total_size(files: &[(String, u64)]) -> Result<u64, String> {
    let total = files
        .iter()
        .try_fold(0u64, |total, (_, size)| total.checked_add(*size))
        .ok_or_else(|| "Repo modèle trop volumineux".to_string())?;
    if total == 0 || total > MAX_MODEL_BYTES {
        return Err("Repo modèle trop volumineux".into());
    }
    Ok(total)
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
    expected_size: u64,
    downloaded: &mut u64,
    total_size: u64,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
    last_percent_sent: &mut f64,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| "Download échoué".to_string())?;
    let resp = tokio::time::timeout(std::time::Duration::from_secs(30), client.get(url).send())
        .await
        .map_err(|_| "Download expiré".to_string())?
        .map_err(|_| "Download échoué".to_string())?;

    if !resp.status().is_success() {
        return Err("Download erreur".into());
    }

    if resp
        .content_length()
        .is_some_and(|size| size > expected_size)
    {
        return Err("Download trop volumineux".into());
    }
    download_io::write_model_response(
        resp,
        dest,
        expected_size,
        downloaded,
        total_size,
        cancel,
        on_progress,
        last_percent_sent,
        MIN_PROGRESS_STEP,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::{checked_total_size, MAX_MODEL_BYTES};

    #[test]
    fn declared_model_size_is_bounded_and_overflow_safe() {
        assert_eq!(checked_total_size(&[("a".into(), 10)]).unwrap(), 10);
        assert!(checked_total_size(&[("a".into(), MAX_MODEL_BYTES + 1)]).is_err());
        assert!(checked_total_size(&[("a".into(), u64::MAX), ("b".into(), 1)]).is_err());
    }
}
