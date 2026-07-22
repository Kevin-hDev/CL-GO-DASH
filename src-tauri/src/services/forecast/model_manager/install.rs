use super::{
    download, download_github,
    install_plan::{install_plan, should_remove_prepared_runtime, InstallPlan},
    models_dir,
    runtime_install::prepare_runtime,
    sidecar_dir, smoke,
};
use crate::services::forecast::{catalog, sidecar_runtime, validation};
use crate::services::model_downloads::{ModelDownloadPhase, ProgressUpdate};
use std::path::Path;
use tokio_util::sync::CancellationToken;

const DOWNLOAD_SHARE: u16 = 70;

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
    if catalog::requires_remote_code(model_id) {
        return Err("Modèle Forecast désactivé par la politique de sécurité".into());
    }
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
            download::download_model(
                repo,
                spec.hf_revision,
                catalog::hf_file(model_id),
                &staging,
                cancel,
                &scaled,
            )
            .await
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

    let runtime_python = match prepare_runtime(sidecar, spec.family_id, cancel, on_progress).await {
        Ok(python) => python,
        Err(error) => {
            if plan == InstallPlan::Full {
                let _ = tokio::fs::remove_dir_all(&staging).await;
            }
            return Err(error);
        }
    };
    let smoke_target = if plan == InstallPlan::Full {
        &staging
    } else {
        &target
    };
    if let Err(error) = smoke::validate_model(
        &runtime_python,
        sidecar,
        models,
        smoke_target,
        spec.family_id,
        cancel,
    )
    .await
    {
        if plan == InstallPlan::Full {
            let _ = tokio::fs::remove_dir_all(&staging).await;
        }
        remove_unvalidated_runtime(runtime_was_ready, sidecar, spec.family_id).await;
        return Err(error);
    }
    if matches!(plan, InstallPlan::RuntimeOnly | InstallPlan::Validate) {
        return Ok(());
    }
    let result = finalize_model(&staging, &target, cancel, on_progress).await;
    if result.is_err() {
        remove_orphan_runtime(runtime_was_ready, models, sidecar, spec.family_id).await;
    }
    result
}

async fn remove_unvalidated_runtime(runtime_was_ready: bool, sidecar: &Path, family_id: &str) {
    if runtime_was_ready {
        return;
    }
    let directory = sidecar.to_path_buf();
    let family = family_id.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        sidecar_runtime::remove_family_runtime(&directory, &family)
    })
    .await;
}

async fn remove_orphan_runtime(
    runtime_was_ready: bool,
    models: &Path,
    sidecar: &Path,
    family_id: &str,
) {
    if !should_remove_prepared_runtime(runtime_was_ready, models, family_id) {
        return;
    }
    let directory = sidecar.to_path_buf();
    let family = family_id.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        sidecar_runtime::remove_family_runtime(&directory, &family)
    })
    .await;
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
