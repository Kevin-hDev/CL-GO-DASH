use super::{auth, catalog};
use crate::services::oauth_providers::ProviderId;
use subtle::ConstantTimeEq;

#[test]
fn provider_catalog_never_duplicates_native_tools() {
    let kimi = catalog::allowed_names(ProviderId::Moonshot);
    let grok = catalog::allowed_names(ProviderId::Xai);

    assert!(!kimi.contains(&"bash"));
    assert!(grok.contains(&"bash"));
    for forbidden in ["read_file", "web_search", "create_branch", "delegate_task"] {
        assert!(!kimi.contains(&forbidden));
        assert!(!grok.contains(&forbidden));
    }
    for shared in ["search_mcp_tools", "forecast", "read_spreadsheet"] {
        assert!(kimi.contains(&shared));
        assert!(grok.contains(&shared));
    }
}

#[test]
fn bearer_auth_rejects_wrong_or_malformed_values() {
    let expected = "a".repeat(32);
    let valid = format!("Bearer {expected}");
    let wrong = format!("Bearer {}b", "a".repeat(31));
    assert!(auth::valid_bearer(&valid, &expected));
    assert!(!auth::valid_bearer(&wrong, &expected));
    assert!(!auth::valid_bearer(&expected, &expected));
    assert!(!auth::valid_bearer("Bearer short", &expected));
}

#[test]
fn oversized_http_body_is_rejected() {
    let request = format!(
        "POST /mcp HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
        super::http::MAX_BODY_BYTES + 1
    );
    assert!(super::http::parse_head(request.as_bytes()).is_err());
}

#[test]
fn session_tokens_are_random_and_fixed_size() {
    let first = auth::generate_token();
    let second = auth::generate_token();
    assert_eq!(first.len(), 43);
    assert_eq!(second.len(), 43);
    assert!(!bool::from(first.as_bytes().ct_eq(second.as_bytes())));
}

#[test]
fn tool_output_is_bounded_on_utf8_boundaries() {
    let value = super::execute::truncate("é".repeat(1024 * 1024));
    assert!(value.len() <= 1024 * 1024);
    assert!(std::str::from_utf8(value.as_bytes()).is_ok());
}
