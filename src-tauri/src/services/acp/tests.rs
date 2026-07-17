use super::{auth_method, native_tool_allowed, AcpConnection, AcpUpdate, JsonLineReader};
use crate::services::oauth_providers::ProviderId;
use serde_json::json;
use tokio::io::{AsyncWriteExt, DuplexStream};

#[tokio::test]
async fn reads_fragmented_and_parallel_json_lines() {
    let (reader, mut writer) = tokio::io::duplex(256);
    let task = tokio::spawn(async move {
        writer.write_all(b"{\"id\":1,\"res").await.unwrap();
        writer
            .write_all(b"ult\":{}}\n{\"method\":\"session/update\"}\n")
            .await
            .unwrap();
    });
    let mut lines = JsonLineReader::new(reader);

    assert_eq!(lines.next_value().await.unwrap()["id"], 1);
    assert_eq!(
        lines.next_value().await.unwrap()["method"],
        "session/update"
    );
    task.await.unwrap();
}

#[tokio::test]
async fn rejects_an_oversized_message_before_parsing() {
    let (reader, mut writer): (DuplexStream, DuplexStream) = tokio::io::duplex(2 * 1024 * 1024);
    let task = tokio::spawn(async move {
        writer
            .write_all(&vec![b'a'; 1024 * 1024 + 1])
            .await
            .unwrap();
    });
    let mut lines = JsonLineReader::new(reader);

    assert!(lines.next_value().await.is_err());
    task.await.unwrap();
}

#[test]
fn maps_known_and_unknown_session_updates() {
    let message = json!({"params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"Bonjour"}}}});
    assert_eq!(
        AcpUpdate::from_message(&message),
        AcpUpdate::Text("Bonjour".into())
    );

    let unknown = json!({"params":{"update":{"sessionUpdate":"future_event","value":42}}});
    assert!(matches!(
        AcpUpdate::from_message(&unknown),
        AcpUpdate::Unknown(kind) if kind == "future_event"
    ));
}

#[test]
fn ignores_protocol_updates_that_are_not_user_visible_activities() {
    for kind in ["available_commands_update", "user_message_chunk"] {
        let message = json!({"params":{"update":{"sessionUpdate":kind}}});
        assert_eq!(AcpUpdate::from_message(&message), AcpUpdate::Ignored);
    }
}

#[test]
fn distinguishes_native_and_cl_go_tool_sources() {
    let native = json!({"params":{"update":{
        "sessionUpdate":"tool_call", "toolCallId":"one", "title":"Bash",
        "kind":"execute", "rawInput":{"command":"pwd"}
    }}});
    let mcp = json!({"params":{"update":{
        "sessionUpdate":"tool_call", "toolCallId":"two", "title":"bash",
        "kind":"execute", "rawInput":{"command":"pwd"}
    }}});

    assert!(matches!(AcpUpdate::from_message(&native),
        AcpUpdate::ToolCall { source, .. } if source == "native"));
    assert!(matches!(AcpUpdate::from_message(&mcp),
        AcpUpdate::ToolCall { source, .. } if source == "mcp"));
}

#[test]
fn grok_native_bash_is_denied_even_if_it_leaks_through() {
    assert!(!native_tool_allowed(ProviderId::Xai, "Bash"));
    assert!(!native_tool_allowed(ProviderId::Xai, "bash"));
    assert!(native_tool_allowed(ProviderId::Xai, "Read"));
    assert!(native_tool_allowed(ProviderId::Moonshot, "Bash"));
}

#[test]
fn auth_selection_is_provider_specific() {
    let methods = json!([{"id":"login"},{"id":"cached_token"},{"id":"xai.api_key"}]);
    assert_eq!(auth_method(ProviderId::Moonshot, &methods), Some("login"));
    assert_eq!(auth_method(ProviderId::Xai, &methods), Some("cached_token"));
}

#[test]
fn oauth_agents_never_hijack_api_provider_ids() {
    assert_eq!(
        super::provider_from_chat("moonshot-oauth"),
        Some(ProviderId::Moonshot)
    );
    assert_eq!(
        super::provider_from_chat("xai-oauth"),
        Some(ProviderId::Xai)
    );
    assert_eq!(super::provider_from_chat("moonshot"), None);
    assert_eq!(super::provider_from_chat("xai"), None);
}

#[tokio::test]
async fn correlates_a_fragmented_json_rpc_response() {
    let (client_stream, server_stream) = tokio::io::duplex(4096);
    let (client_read, client_write) = tokio::io::split(client_stream);
    let (server_read, mut server_write) = tokio::io::split(server_stream);
    let server = tokio::spawn(async move {
        let mut reader = JsonLineReader::new(server_read);
        let request = reader.next_value().await.unwrap();
        assert_eq!(request["method"], "initialize");
        let id = request["id"].as_str().unwrap();
        let response =
            format!("{{\"jsonrpc\":\"2.0\",\"id\":\"{id}\",\"result\":{{\"authMethods\":[]}}}}\n");
        let middle = response.len() / 2;
        server_write
            .write_all(&response.as_bytes()[..middle])
            .await
            .unwrap();
        server_write
            .write_all(&response.as_bytes()[middle..])
            .await
            .unwrap();
    });
    let mut connection = AcpConnection::new(client_read, client_write);

    let response = connection
        .request("initialize", json!({"protocolVersion": 1}))
        .await
        .unwrap();

    assert_eq!(response["authMethods"], json!([]));
    server.await.unwrap();
}

#[tokio::test]
async fn stores_only_bounded_opaque_session_metadata() {
    let cl_go_session = uuid::Uuid::new_v4().to_string();
    super::session_store::save(&cl_go_session, ProviderId::Moonshot, "opaque-session")
        .await
        .unwrap();

    assert_eq!(
        super::session_store::load(&cl_go_session, ProviderId::Moonshot)
            .await
            .unwrap()
            .as_deref(),
        Some("opaque-session"),
    );
    assert_eq!(
        super::session_store::load(&cl_go_session, ProviderId::Xai)
            .await
            .unwrap(),
        None
    );
    assert!(
        super::session_store::save(&cl_go_session, ProviderId::Moonshot, &"x".repeat(300))
            .await
            .is_err()
    );
    super::session_store::remove(&cl_go_session).await.unwrap();
}

#[tokio::test]
#[ignore = "requires the official Kimi client"]
async fn official_kimi_client_negotiates_acp_v1_and_http_mcp() {
    assert!(super::probe(ProviderId::Moonshot).await);
}

#[tokio::test]
#[ignore = "requires the official Grok client"]
async fn official_grok_client_negotiates_acp_v1_and_http_mcp() {
    assert!(super::probe(ProviderId::Xai).await);
}
