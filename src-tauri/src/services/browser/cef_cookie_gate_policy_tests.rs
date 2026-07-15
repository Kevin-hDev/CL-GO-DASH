#[test]
fn security_gate_allows_time_for_system_approval() {
    let source = include_str!("cef_cookie_gate.rs");

    assert!(source.contains("Duration::from_secs(60)"));
}
