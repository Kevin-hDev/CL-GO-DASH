use crate::services::forecast::validation;
use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};

pub mod download;
pub mod download_github;
mod install;
mod install_plan;
mod runtime_install;
mod smoke;
mod uninstall;

pub use install::install_with_callback;
pub use uninstall::uninstall;

fn models_dir() -> PathBuf {
    data_dir().join("forecast-models")
}

fn sidecar_dir() -> PathBuf {
    data_dir().join("forecast-sidecar")
}

pub fn model_path(model_id: &str) -> PathBuf {
    models_dir().join(model_id)
}

pub fn is_installed(model_id: &str) -> bool {
    validation::validate_model_id(model_id).is_ok() && is_installed_in(&models_dir(), model_id)
}

pub fn is_ready(model_id: &str) -> bool {
    let Some(spec) = crate::services::forecast::catalog::find_model(model_id) else {
        return false;
    };
    is_installed(model_id)
        && crate::services::forecast::sidecar_runtime::family_runtime_ready(
            &sidecar_dir(),
            spec.family_id,
        )
        && smoke::is_validated(&model_path(model_id), &sidecar_dir())
}

pub fn get_model_size(model_id: &str) -> u64 {
    if validation::validate_model_id(model_id).is_err() {
        return 0;
    }
    walkdir_size(&model_path(model_id))
}

fn is_installed_in(root: &Path, model_id: &str) -> bool {
    has_downloaded_weights(root, model_id)
}

fn has_downloaded_weights(root: &Path, model_id: &str) -> bool {
    root.join(model_id).join(".complete").is_file()
}

fn family_has_installed_model(models: &Path, family_id: &str) -> bool {
    crate::services::forecast::catalog::FORECAST_MODELS
        .iter()
        .filter(|model| !model.is_cloud && model.family_id == family_id)
        .any(|model| is_installed_in(models, model.id))
}

fn walkdir_size(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| {
                    if entry.path().is_dir() {
                        walkdir_size(&entry.path())
                    } else {
                        entry.metadata().map(|meta| meta.len()).unwrap_or(0)
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
