use crate::services::forecast::validation;
use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};

pub mod download;
pub mod download_github;
mod install;
mod runtime_install;
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

pub fn get_model_size(model_id: &str) -> u64 {
    if validation::validate_model_id(model_id).is_err() {
        return 0;
    }
    walkdir_size(&model_path(model_id))
}

fn is_installed_in(root: &Path, model_id: &str) -> bool {
    root.join(model_id).join(".complete").exists()
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
