use super::config::{self, StoredConnector, MAX_CONNECTORS};
use super::trusted::is_trusted_endpoint_for_connector;

fn connector(id: &str) -> StoredConnector {
    StoredConnector {
        id: id.to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: Some("https://mcp.notion.com/mcp".to_string()),
        install_command: None,
        env_keys: None,
    }
}

#[test]
fn validates_catalog_endpoint_for_matching_connector() {
    assert!(is_trusted_endpoint_for_connector(
        "notion",
        "https://mcp.notion.com/mcp"
    ));
}

#[test]
fn rejects_catalog_endpoint_for_wrong_connector() {
    let mut c = connector("sentry");
    c.endpoint = Some("https://mcp.notion.com/mcp".to_string());
    assert!(config::validate_connector(&c).is_err());
}

#[test]
fn rejects_invalid_status() {
    let mut c = connector("notion");
    c.status = "pending".to_string();
    assert!(config::validate_connector(&c).is_err());
}

#[test]
fn max_connector_limit_is_bounded() {
    assert_eq!(MAX_CONNECTORS, 32);
}

#[test]
fn imessage_install_command_is_forced() {
    let c = StoredConnector {
        id: "imessage".to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: None,
        install_command: Some("npx bad".to_string()),
        env_keys: None,
    };
    let cmd = config::install_command_for(&c).unwrap();
    assert!(cmd.contains("jsr:@wyattjoh/imessage-mcp"));
    assert!(!cmd.contains("npx bad"));
}
