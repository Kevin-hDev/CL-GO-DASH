#[test]
fn build_script_never_embeds_dotenv_values_in_the_binary() {
    let build =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("build.rs"))
            .expect("Rust build script");

    assert!(!build.contains("load_env"));
    assert!(!build.contains("cargo:rustc-env"));
}
