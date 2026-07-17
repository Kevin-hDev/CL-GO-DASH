use super::{login_diagnostics::LoginDiagnostic, ProviderId};

#[test]
fn diagnostic_ids_must_be_random_uuids() {
    let id = uuid::Uuid::new_v4().to_string();
    assert!(LoginDiagnostic::from_ui(ProviderId::Moonshot, &id).is_ok());
    assert!(LoginDiagnostic::from_ui(ProviderId::Moonshot, "not-a-uuid").is_err());
    assert!(LoginDiagnostic::from_ui(
        ProviderId::Moonshot,
        "00000000-0000-0000-0000-000000000000\nsecret=leak"
    )
    .is_err());
}
