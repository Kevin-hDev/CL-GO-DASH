use crate::services::forecast::types::ModelDownloadProgress;
use std::io::Cursor;
use std::path::{Component, Path};
use tauri::ipc::Channel;

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
        .map_err(|e| format!("Téléchargement GitHub échoué: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub erreur: {}", resp.status()));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Archive GitHub invalide: {e}"))?;

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
        zip::ZipArchive::new(reader).map_err(|e| format!("Zip GitHub invalide: {e}"))?;

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|e| format!("Zip GitHub illisible: {e}"))?;
        let Some(path) = file.enclosed_name().map(|p| p.to_path_buf()) else {
            continue;
        };
        let stripped = path
            .components()
            .skip(1)
            .collect::<std::path::PathBuf>();
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
            std::fs::create_dir_all(&dest).map_err(|e| format!("Dossier GitHub: {e}"))?;
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Parent GitHub: {e}"))?;
        }
        let mut out = std::fs::File::create(&dest).map_err(|e| format!("Fichier GitHub: {e}"))?;
        std::io::copy(&mut file, &mut out).map_err(|e| format!("Copie GitHub: {e}"))?;
    }
    Ok(())
}
