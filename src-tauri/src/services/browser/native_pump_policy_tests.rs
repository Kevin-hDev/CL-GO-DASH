#[test]
fn native_pump_uses_a_main_thread_owner_instead_of_manual_send_sync() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/services/browser/native_pump_wake.rs"),
    )
    .expect("native pump source");

    assert!(source.contains("MainThreadBound"));
    assert!(!source.contains("unsafe impl Send for PumpWake"));
    assert!(!source.contains("unsafe impl Sync for PumpWake"));
}
