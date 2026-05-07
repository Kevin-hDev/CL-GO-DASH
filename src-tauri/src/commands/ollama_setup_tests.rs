use crate::commands::ollama_bundle_utils::is_valid_semver;

#[test]
fn valid_semver_accepted() {
    assert!(is_valid_semver("0.23.1"));
    assert!(is_valid_semver("1.0.0"));
    assert!(is_valid_semver("0.30.0-rc3"));
    assert!(is_valid_semver("2.1.0-beta.1"));
}

#[test]
fn invalid_semver_rejected() {
    assert!(!is_valid_semver(""));
    assert!(!is_valid_semver("1.0"));
    assert!(!is_valid_semver("abc"));
    assert!(!is_valid_semver("1.0.0/../../evil"));
    assert!(!is_valid_semver("1.0.0%0d%0aHeader: inject"));
    assert!(!is_valid_semver("1.0.0\nmalicious"));
    assert!(!is_valid_semver("v1.0.0"));
}
