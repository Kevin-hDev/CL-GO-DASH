#[test]
fn build_script_never_embeds_dotenv_values_in_the_binary() {
    let build =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("build.rs"))
            .expect("Rust build script");

    assert!(!build.contains("load_env"));
    assert!(!build.contains("cargo:rustc-env"));
}

#[test]
fn native_runtime_modules_are_not_built_in_linux_library() {
    let module = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/services/browser/mod.rs"),
    )
    .expect("browser module");

    for runtime_module in [
        "browser_view_key",
        "cookie_store_probe",
        "lifecycle",
        "native_paths",
        "navigation_target",
        "runtime_revision",
        "session_model_runtime",
        "view_recency",
        "view_state",
    ] {
        let guarded = format!(
            "#[cfg(any(test, target_os = \"macos\", target_os = \"windows\"))]\nmod {runtime_module};"
        );
        assert!(
            module.contains(&guarded),
            "{runtime_module} must be excluded from the Linux library build"
        );
    }
}
