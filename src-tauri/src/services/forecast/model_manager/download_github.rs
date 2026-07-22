use crate::services::model_downloads::{ModelDownloadPhase, ProgressUpdate};
use futures_util::StreamExt;
use std::io::Cursor;
use std::path::{Component, Path};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

const MAX_GITHUB_ARCHIVE_BYTES: u64 = 200 * 1024 * 1024;
const MAX_GITHUB_EXTRACTED_BYTES: u64 = 500 * 1024 * 1024;
const MAX_GITHUB_FILES: usize = 20_000;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);
const CHUNK_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn download_repo_snapshot(
    repo: &str,
    revision: Option<&str>,
    target_dir: &Path,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
) -> Result<(), String> {
    let rev = revision.unwrap_or("main");
    let url = format!("https://codeload.github.com/{repo}/zip/{rev}");
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| "Téléchargement GitHub échoué".to_string())?;
    let resp = tokio::time::timeout(
        RESPONSE_TIMEOUT,
        client
            .get(&url)
            .header("User-Agent", "CL-GO-DASH/1.0")
            .send(),
    )
    .await
    .map_err(|_| "Téléchargement GitHub expiré".to_string())?
    .map_err(|_| "Téléchargement GitHub échoué".to_string())?;

    if !resp.status().is_success() {
        return Err("GitHub erreur".into());
    }
    if resp
        .content_length()
        .is_some_and(|size| size > MAX_GITHUB_ARCHIVE_BYTES)
    {
        return Err("Archive GitHub trop volumineuse".into());
    }

    let total = resp.content_length().unwrap_or_default();
    let mut downloaded = 0u64;
    let mut bytes = Vec::new();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = tokio::time::timeout(CHUNK_TIMEOUT, stream.next())
        .await
        .map_err(|_| "Téléchargement GitHub expiré".to_string())?
    {
        if cancel.is_cancelled() {
            return Err("cancelled".into());
        }
        let chunk = chunk.map_err(|_| "Archive GitHub invalide".to_string())?;
        downloaded = downloaded.saturating_add(chunk.len() as u64);
        if downloaded > MAX_GITHUB_ARCHIVE_BYTES {
            return Err("Archive GitHub trop volumineuse".into());
        }
        bytes.extend_from_slice(&chunk);
        let percent = if total > 0 {
            ((downloaded as f64 / total as f64) * 100.0).round() as u8
        } else {
            0
        };
        on_progress(ProgressUpdate {
            phase: ModelDownloadPhase::Downloading,
            downloaded,
            total,
            percent: percent.min(99),
        });
    }

    on_progress(ProgressUpdate {
        phase: ModelDownloadPhase::Installing,
        downloaded,
        total,
        percent: 99,
    });
    let target = target_dir.to_path_buf();
    let cancel_clone = cancel.clone();
    tokio::task::spawn_blocking(move || extract_repo_zip(&bytes, &target, &cancel_clone))
        .await
        .map_err(|_| "Archive GitHub invalide".to_string())?
}

fn extract_repo_zip(
    bytes: &[u8],
    target_dir: &Path,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let reader = Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|_| "Zip GitHub invalide".to_string())?;
    if archive.len() > MAX_GITHUB_FILES {
        return Err("Archive GitHub trop volumineuse".into());
    }

    let mut extracted_bytes = 0u64;
    for index in 0..archive.len() {
        if cancel.is_cancelled() {
            return Err("cancelled".into());
        }
        let mut file = archive
            .by_index(index)
            .map_err(|_| "Zip GitHub illisible".to_string())?;
        extracted_bytes = extracted_bytes.saturating_add(file.size());
        if extracted_bytes > MAX_GITHUB_EXTRACTED_BYTES {
            return Err("Archive GitHub trop volumineuse".into());
        }
        let Some(path) = file.enclosed_name().map(|p| p.to_path_buf()) else {
            continue;
        };
        let stripped = path.components().skip(1).collect::<std::path::PathBuf>();
        if stripped.as_os_str().is_empty() {
            continue;
        }
        if !stripped
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err("Archive GitHub invalide".into());
        }

        let dest = target_dir.join(stripped);
        if file.is_dir() {
            std::fs::create_dir_all(&dest).map_err(|_| "Dossier GitHub invalide".to_string())?;
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|_| "Dossier GitHub invalide".to_string())?;
        }
        let mut out =
            std::fs::File::create(&dest).map_err(|_| "Fichier GitHub invalide".to_string())?;
        std::io::copy(&mut file, &mut out).map_err(|_| "Copie GitHub échouée".to_string())?;
    }
    Ok(())
}
