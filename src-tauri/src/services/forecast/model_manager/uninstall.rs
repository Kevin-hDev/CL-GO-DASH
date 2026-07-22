use super::{family_has_installed_model, models_dir, sidecar_dir};
use crate::services::forecast::{catalog, sidecar_runtime, validation};
use std::path::Path;

pub async fn uninstall(model_id: &str) -> Result<(), String> {
    uninstall_from_roots(model_id, &models_dir(), &sidecar_dir()).await
}

pub(super) async fn uninstall_from_roots(
    model_id: &str,
    models: &Path,
    sidecar: &Path,
) -> Result<(), String> {
    validation::validate_model_id(model_id)?;
    let spec =
        catalog::find_model(model_id).ok_or_else(|| "Modèle Forecast inconnu".to_string())?;
    let model_path = models.join(model_id);
    if model_path.exists() {
        tokio::fs::remove_dir_all(&model_path)
            .await
            .map_err(|_| "Suppression du modèle Forecast impossible".to_string())?;
    }
    let staging = models.join(format!(".{model_id}.staging"));
    if staging.exists() {
        tokio::fs::remove_dir_all(staging)
            .await
            .map_err(|_| "Suppression du modèle Forecast impossible".to_string())?;
    }
    if family_has_installed_model(models, spec.family_id) {
        return Ok(());
    }
    let directory = sidecar.to_path_buf();
    let family = spec.family_id.to_string();
    tokio::task::spawn_blocking(move || sidecar_runtime::remove_family_runtime(&directory, &family))
        .await
        .map_err(|_| "Suppression du runtime Forecast impossible".to_string())?
}
