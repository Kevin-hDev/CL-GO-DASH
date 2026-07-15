use serde_json::Value;

#[test]
fn windows_bundle_stages_the_sandboxed_cef_runtime_at_the_app_root() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let config: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("tauri.windows.conf.json"))
            .expect("windows bundle config"),
    )
    .expect("valid windows bundle config");

    let hook = config
        .pointer("/build/beforeBundleCommand")
        .and_then(Value::as_str)
        .expect("Windows CEF bundle hook");
    assert!(hook.contains("prepare-cef-windows.ps1"));
    assert_eq!(
        config
            .pointer("/bundle/resources/target~1cef-runtime~1windows~1")
            .and_then(Value::as_str),
        Some("")
    );
}

#[test]
fn windows_bundle_hook_pins_and_verifies_the_cef_bootstrap() {
    let script = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/prepare-cef-windows.ps1"),
    )
    .expect("Windows CEF bundle hook");

    assert!(script.contains("150.0.10+g8042e43"));
    assert!(script.contains("bce95ec52696c6725447fd0bf993cc928aefecd4"));
    assert!(script.contains("eab5d939293a666b210b8f5faec191324a017d6105485cfc45150863607bd367"));
    assert!(script.contains("cl-go-dash.dll"));
    assert!(script.contains("LICENSE.txt"));
    assert!(script.contains("CREDITS.html"));
}
