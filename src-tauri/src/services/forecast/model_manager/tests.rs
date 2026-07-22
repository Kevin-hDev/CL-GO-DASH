use super::install::{install_plan, should_remove_prepared_runtime, InstallPlan};
use super::uninstall::uninstall_from_roots;
use std::fs;

fn mark_installed(models: &std::path::Path, model_id: &str) {
    let model = models.join(model_id);
    fs::create_dir_all(&model).unwrap();
    fs::write(model.join(".complete"), "ok").unwrap();
}

#[test]
fn installed_weights_only_require_the_missing_runtime() {
    let temp = tempfile::tempdir().unwrap();
    let models = temp.path().join("models");
    let sidecar = temp.path().join("sidecar");
    fs::create_dir_all(&sidecar).unwrap();
    fs::write(sidecar.join("requirements.txt"), "numpy\n").unwrap();
    mark_installed(&models, "chronos-bolt-tiny");

    assert_eq!(
        install_plan(&models, &sidecar, "chronos-bolt-tiny").unwrap(),
        InstallPlan::RuntimeOnly
    );
}

#[test]
fn orphan_cleanup_preserves_preexisting_and_shared_runtimes() {
    let temp = tempfile::tempdir().unwrap();
    let models = temp.path().join("models");

    assert!(should_remove_prepared_runtime(
        false,
        &models,
        "chronos-bolt"
    ));
    assert!(!should_remove_prepared_runtime(
        true,
        &models,
        "chronos-bolt"
    ));

    mark_installed(&models, "chronos-bolt-mini");
    assert!(!should_remove_prepared_runtime(
        false,
        &models,
        "chronos-bolt"
    ));
}

#[tokio::test]
async fn shared_runtime_is_removed_only_after_the_last_model() {
    let temp = tempfile::tempdir().unwrap();
    let models = temp.path().join("models");
    let sidecar = temp.path().join("sidecar");
    let runtime = sidecar.join(".venvs").join("chronos-bolt");
    mark_installed(&models, "chronos-bolt-tiny");
    mark_installed(&models, "chronos-bolt-mini");
    fs::create_dir_all(&runtime).unwrap();

    uninstall_from_roots("chronos-bolt-tiny", &models, &sidecar)
        .await
        .unwrap();
    assert!(runtime.exists());

    uninstall_from_roots("chronos-bolt-mini", &models, &sidecar)
        .await
        .unwrap();
    assert!(!runtime.exists());
}
