use crate::services::forecast::types::ModelDownloadProgress;
use std::io::Cursor;
use std::path::{Component, Path};
use tauri::ipc::Channel;

const MAX_GITHUB_ARCHIVE_BYTES: u64 = 200 * 1024 * 1024;
const MAX_GITHUB_EXTRACTED_BYTES: u64 = 500 * 1024 * 1024;
const MAX_GITHUB_FILES: usize = 20_000;

pub async fn download_repo_snapshot(
    repo: &str,
    revision: Option<&str>,
    target_dir: &Path,
    model_id: &str,
    on_progress: &Channel<ModelDownloadProgress>,
) -> Result<(), String> {
    let rev = revision.unwrap_or("main");
    let url = format!("https://codeload.github.com/{repo}/zip/refs/heads/{rev}");
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "CL-GO-DASH/1.0")
        .send()
        .await
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

    let bytes = resp
        .bytes()
        .await
        .map_err(|_| "Archive GitHub invalide".to_string())?;
    if bytes.len() as u64 > MAX_GITHUB_ARCHIVE_BYTES {
        return Err("Archive GitHub trop volumineuse".into());
    }

    let _ = on_progress.send(ModelDownloadProgress {
        model_name: model_id.to_string(),
        downloaded: bytes.len() as u64,
        total: bytes.len() as u64,
        percent: 100.0,
    });

    extract_repo_zip(&bytes, target_dir)
}

fn extract_repo_zip(bytes: &[u8], target_dir: &Path) -> Result<(), String> {
    let reader = Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|_| "Zip GitHub invalide".to_string())?;
    if archive.len() > MAX_GITHUB_FILES {
        return Err("Archive GitHub trop volumineuse".into());
    }

    let mut extracted_bytes = 0u64;
    for index in 0..archive.len() {
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
