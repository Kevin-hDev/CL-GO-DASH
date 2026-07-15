use std::path::Path;

const HELPERS: &[&str] = &[
    "CL-GO Helper",
    "CL-GO Helper (GPU)",
    "CL-GO Helper (Renderer)",
    "CL-GO Helper (Plugin)",
    "CL-GO Helper (Alerts)",
];

#[test]
fn every_cef_helper_has_a_bounded_valid_bundle_manifest() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/cef/macos/helpers");

    for helper in HELPERS {
        let manifest = root.join(format!("{helper}.app/Contents/Info.plist"));
        let content = std::fs::read_to_string(manifest).expect("helper Info.plist");
        assert!(content.len() < 8_192);
        assert!(content.contains("<key>CFBundleExecutable</key>"));
        assert!(content.contains(&format!("<string>{helper}</string>")));
        assert!(content.contains("<key>LSUIElement</key>"));
        assert!(!content.contains("PLACEHOLDER"));
    }
}

#[test]
fn unsigned_release_uses_a_verifiable_adhoc_bundle_signature() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let content = std::fs::read_to_string(path).expect("tauri config");
    assert!(content.len() < 65_536);
    let config: serde_json::Value = serde_json::from_str(&content).expect("valid tauri config");
    let macos = config
        .pointer("/bundle/macOS")
        .expect("macOS bundle config");

    assert_eq!(
        macos
            .get("signingIdentity")
            .and_then(|value| value.as_str()),
        Some("-")
    );
    assert_eq!(
        macos
            .get("hardenedRuntime")
            .and_then(|value| value.as_bool()),
        Some(false)
    );
}

#[test]
fn development_command_starts_vite_without_waiting_for_cef() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let content = std::fs::read_to_string(path).expect("tauri config");
    let config: serde_json::Value = serde_json::from_str(&content).expect("valid tauri config");
    let command = config
        .pointer("/build/beforeDevCommand")
        .and_then(|value| value.as_str())
        .expect("beforeDevCommand");

    assert_eq!(command, "npm run dev");
}

#[test]
fn macos_development_runner_uses_a_real_application_bundle() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_config = std::fs::read_to_string(root.join(".cargo/config.toml"))
        .expect("macOS development runner config");
    let runner = std::fs::read_to_string(root.join("scripts/run-cef-dev-app.sh"))
        .expect("macOS development runner");

    assert!(cargo_config.len() < 8_192);
    assert!(cargo_config.contains("runner = \"bash scripts/run-cef-dev-app.sh\""));
    assert!(runner.len() < 16_384);
    let preparation = runner
        .find("bash scripts/ensure-cef-dev-runtime.sh")
        .expect("cached CEF preparation");
    let bundle = runner
        .find("CL-GO Dev.app/Contents/MacOS")
        .expect("development application bundle");
    assert!(preparation < bundle);
    assert!(runner.contains("CL-GO Dev.app/Contents/MacOS"));
    assert!(runner.contains("exec \"$APP_EXECUTABLE\""));
    assert!(runner.contains("exec \"$BINARY\" \"$@\""));
    assert!(!runner.contains("--no-sandbox"));
}
