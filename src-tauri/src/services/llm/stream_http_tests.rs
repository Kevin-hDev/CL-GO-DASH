use std::time::Duration;

use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

#[tokio::test]
async fn chat_request_refuses_redirects_before_forwarding_the_body() {
    let destination = MockServer::start().await;
    let origin = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("Location", format!("{}/sink", destination.uri())),
        )
        .mount(&origin)
        .await;

    let client = crate::services::secure_http::AuthenticatedClient::new(Duration::from_secs(2))
        .expect("client");
    let result = send_json_request(
        &client,
        &format!("{}/chat", origin.uri()),
        "fixture-secret",
        &serde_json::json!({"private": "payload"}),
    )
    .await;

    assert!(result.is_err());
    assert!(destination
        .received_requests()
        .await
        .expect("requests")
        .is_empty());
}

#[tokio::test]
async fn oversized_provider_error_is_not_loaded() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(500).set_body_string(
                "x".repeat(crate::services::secure_http::PROVIDER_ERROR_LIMIT + 1),
            ),
        )
        .mount(&server)
        .await;
    let client = crate::services::secure_http::AuthenticatedClient::new(Duration::from_secs(2))
        .expect("client");
    let response = client
        .send(client.get(server.uri()))
        .await
        .expect("response");

    let body = read_provider_error(response).await;

    assert!(body.is_empty());
}
