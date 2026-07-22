use super::{
    download, download_github, family_has_installed_model, is_installed_in, models_dir,
    runtime_install::prepare_runtime, sidecar_dir,
};
use crate::services::forecast::{catalog, sidecar_runtime, validation};
use crate::services::model_downloads::{ModelDownloadPhase, ProgressUpdate};
use std::path::Path;
use tokio_util::sync::CancellationToken;

const DOWNLOAD_SHARE: u16 = 70;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InstallPlan {
    Full,
    RuntimeOnly,
    Ready,
}

pub async fn install_with_callback<F>(
    model_id: &str,
    cancel: &CancellationToken,
    on_progress: F,
) -> Result<(), String>
where
    F: Fn(ProgressUpdate) + Send + Sync,
{
    install_with_roots(
        model_id,
        &models_dir(),
        &sidecar_dir(),
        cancel,
        &on_progress,
    )
    .await
}

async fn install_with_roots(
    model_id: &str,
    models: &Path,
    sidecar: &Path,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
) -> Result<(), String> {
    validation::validate_model_id(model_id)?;
    let spec =
        catalog::find_model(model_id).ok_or_else(|| "Modèle Forecast inconnu".to_string())?;
    let plan = install_plan(models, sidecar, model_id)?;
    if plan == InstallPlan::Ready {
        return Ok(());
    }

    let runtime_was_ready = sidecar_runtime::family_runtime_ready(sidecar, spec.family_id);
    let target = models.join(model_id);
    let staging = models.join(format!(".{model_id}.staging"));
    if plan == InstallPlan::Full {
        prepare_model_staging(&staging).await?;
        let scaled = |mut progress: ProgressUpdate| {
            progress.percent = ((u16::from(progress.percent) * DOWNLOAD_SHARE) / 100) as u8;
            on_progress(progress);
        };
        let result = if let Some(repo) = spec.hf_repo {
            download::download_model(repo, spec.hf_revision, &staging, cancel, &scaled).await
        } else if let Some(repo) = spec.github_repo {
            download_github::download_repo_snapshot(
                repo,
                spec.github_revision,
                &staging,
                cancel,
                &scaled,
            )
            .await
        } else {
            Err("Source du modèle Forecast indisponible".to_string())
        };
        if let Err(error) = result {
            let _ = tokio::fs::remove_dir_all(&staging).await;
            return Err(error);
        }
    }

    if let Err(error) = prepare_runtime(sidecar, spec.family_id, cancel, on_progress).await {
        if plan == InstallPlan::Full {
            let _ = tokio::fs::remove_dir_all(&staging).await;
        }
        return Err(error);
    }
    if plan == InstallPlan::RuntimeOnly {
        return Ok(());
    }
    let result = finalize_model(&staging, &target, cancel, on_progress).await;
    if result.is_err() && should_remove_prepared_runtime(runtime_was_ready, models, spec.family_id)
    {
        let directory = sidecar.to_path_buf();
        let family = spec.family_id.to_string();
        let _ = tokio::task::spawn_blocking(move || {
            sidecar_runtime::remove_family_runtime(&directory, &family)
        })
        .await;
    }
    result
}

pub(super) fn should_remove_prepared_runtime(
    runtime_was_ready: bool,
    models: &Path,
    family_id: &str,
) -> bool {
    !runtime_was_ready && !family_has_installed_model(models, family_id)
}

pub(super) fn install_plan(
    models: &Path,
    sidecar: &Path,
    model_id: &str,
) -> Result<InstallPlan, String> {
    let spec =
        catalog::find_model(model_id).ok_or_else(|| "Modèle Forecast inconnu".to_string())?;
    let weights = is_installed_in(models, model_id);
    let runtime = sidecar_runtime::family_runtime_ready(sidecar, spec.family_id);
    Ok(match (weights, runtime) {
        (true, true) => InstallPlan::Ready,
        (true, false) => InstallPlan::RuntimeOnly,
        (false, _) => InstallPlan::Full,
    })
}

async fn prepare_model_staging(staging: &Path) -> Result<(), String> {
    let _ = tokio::fs::remove_dir_all(staging).await;
    tokio::fs::create_dir_all(staging)
        .await
        .map_err(|_| "Impossible de préparer l'installation".to_string())
}

async fn finalize_model(
    staging: &Path,
    target: &Path,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
) -> Result<(), String> {
    if cancel.is_cancelled() {
        let _ = tokio::fs::remove_dir_all(staging).await;
        return Err("cancelled".to_string());
    }
    on_progress(ProgressUpdate {
        phase: ModelDownloadPhase::Installing,
        downloaded: 0,
        total: 0,
        percent: 99,
    });
    tokio::fs::write(staging.join(".complete"), b"ok")
        .await
        .map_err(|_| "Validation installation échouée".to_string())?;
    let _ = tokio::fs::remove_dir_all(target).await;
    tokio::fs::rename(staging, target)
        .await
        .map_err(|_| "Finalisation installation échouée".to_string())
}
