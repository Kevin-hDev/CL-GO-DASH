use super::{
    ensure_runtime, expected_requirements, family_runtime_ready, runtime_paths, RuntimePaths,
};
use std::fs;

fn write_fake_runtime(paths: &RuntimePaths, stamp: &str, marker: &str) {
    let python = paths.python_in(&paths.live);
    fs::create_dir_all(python.parent().unwrap()).unwrap();
    fs::write(python, marker).unwrap();
    fs::write(paths.live.join(".requirements.stamp"), stamp).unwrap();
}

#[test]
fn readiness_requires_the_exact_manifest() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("requirements.txt"), "numpy\n").unwrap();
    let paths = runtime_paths(temp.path(), "chronos-bolt").unwrap();
    write_fake_runtime(&paths, "stale", "python");

    assert!(!family_runtime_ready(temp.path(), "chronos-bolt"));

    let expected = expected_requirements(temp.path(), "chronos-bolt").unwrap();
    fs::write(paths.live.join(".requirements.stamp"), expected).unwrap();
    assert!(family_runtime_ready(temp.path(), "chronos-bolt"));
}

#[test]
fn ensure_runtime_never_installs_missing_dependencies() {
    let temp = tempfile::tempdir().unwrap();

    assert!(ensure_runtime(temp.path(), "chronos-bolt").is_err());
    assert!(!temp.path().join(".venvs").exists());
}

#[test]
fn staged_runtime_replaces_live_runtime_cleanly() {
    let temp = tempfile::tempdir().unwrap();
    let paths = runtime_paths(temp.path(), "chronos-bolt").unwrap();
    write_fake_runtime(&paths, "old", "old-python");
    let staged_python = paths.python_in(&paths.staging);
    fs::create_dir_all(staged_python.parent().unwrap()).unwrap();
    fs::write(&staged_python, "new-python").unwrap();

    super::commit_staged_runtime(&paths).unwrap();

    assert_eq!(
        fs::read_to_string(paths.python_in(&paths.live)).unwrap(),
        "new-python"
    );
    assert!(!paths.staging.exists());
    assert!(!paths.backup.exists());
}

#[test]
fn missing_staging_never_removes_the_live_runtime() {
    let temp = tempfile::tempdir().unwrap();
    let paths = runtime_paths(temp.path(), "chronos-bolt").unwrap();
    write_fake_runtime(&paths, "current", "live-python");

    assert!(super::commit_staged_runtime(&paths).is_err());
    assert_eq!(
        fs::read_to_string(paths.python_in(&paths.live)).unwrap(),
        "live-python"
    );
}
