use crate::services::model_downloads::{ModelDownloadPhase, ProgressUpdate};
use futures_util::StreamExt;
use reqwest::Response;
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

const CHUNK_TIMEOUT: Duration = Duration::from_secs(60);

pub(super) async fn read_bounded_json(
    response: Response,
    maximum: usize,
) -> Result<serde_json::Value, String> {
    if response
        .content_length()
        .is_some_and(|size| size > maximum as u64)
    {
        return Err("Réponse HuggingFace trop volumineuse".into());
    }
    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = tokio::time::timeout(CHUNK_TIMEOUT, stream.next())
        .await
        .map_err(|_| "Réponse HuggingFace expirée".to_string())?
    {
        let chunk = chunk.map_err(|_| "Réponse HuggingFace invalide".to_string())?;
        if body.len().saturating_add(chunk.len()) > maximum {
            return Err("Réponse HuggingFace trop volumineuse".into());
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|_| "Réponse HuggingFace invalide".into())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn write_model_response(
    response: Response,
    destination: &Path,
    expected_size: u64,
    downloaded: &mut u64,
    total_size: u64,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
    last_percent_sent: &mut f64,
    minimum_progress_step: f64,
) -> Result<(), String> {
    let temporary = destination.with_extension("tmp");
    let result = write_stream(
        response,
        &temporary,
        expected_size,
        downloaded,
        total_size,
        cancel,
        on_progress,
        last_percent_sent,
        minimum_progress_step,
    )
    .await;
    if let Err(error) = result {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(error);
    }
    tokio::fs::rename(&temporary, destination)
        .await
        .map_err(|_| "Finalisation du téléchargement impossible".to_string())
}

#[allow(clippy::too_many_arguments)]
async fn write_stream(
    response: Response,
    temporary: &Path,
    expected_size: u64,
    downloaded: &mut u64,
    total_size: u64,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
    last_percent_sent: &mut f64,
    minimum_progress_step: f64,
) -> Result<(), String> {
    let mut file = tokio::fs::File::create(temporary)
        .await
        .map_err(|_| "Création du téléchargement impossible".to_string())?;
    let mut file_size = 0u64;
    let mut stream = response.bytes_stream();
    loop {
        if cancel.is_cancelled() {
            return Err("cancelled".into());
        }
        let next = tokio::time::timeout(CHUNK_TIMEOUT, stream.next())
            .await
            .map_err(|_| "Téléchargement expiré".to_string())?;
        let Some(chunk) = next else { break };
        let chunk = chunk.map_err(|_| "Téléchargement interrompu".to_string())?;
        file_size = file_size
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| "Téléchargement trop volumineux".to_string())?;
        *downloaded = downloaded
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| "Téléchargement trop volumineux".to_string())?;
        if file_size > expected_size || *downloaded > total_size {
            return Err("Téléchargement trop volumineux".into());
        }
        file.write_all(&chunk)
            .await
            .map_err(|_| "Écriture du téléchargement impossible".to_string())?;
        report_progress(
            *downloaded,
            total_size,
            on_progress,
            last_percent_sent,
            minimum_progress_step,
        );
    }
    if file_size != expected_size {
        return Err("Téléchargement incomplet".into());
    }
    file.flush()
        .await
        .map_err(|_| "Validation du téléchargement impossible".to_string())?;
    file.sync_all()
        .await
        .map_err(|_| "Validation du téléchargement impossible".to_string())
}

fn report_progress(
    downloaded: u64,
    total_size: u64,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
    last_percent_sent: &mut f64,
    minimum_progress_step: f64,
) {
    let percent = (downloaded as f64 / total_size as f64) * 100.0;
    if percent >= *last_percent_sent + minimum_progress_step || percent >= 99.9 {
        *last_percent_sent = percent;
        on_progress(ProgressUpdate {
            phase: ModelDownloadPhase::Downloading,
            downloaded,
            total: total_size,
            percent: percent.min(100.0).round() as u8,
        });
    }
}
