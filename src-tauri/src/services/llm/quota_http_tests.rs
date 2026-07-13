use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

#[tokio::test]
async fn authenticated_quota_client_refuses_redirects() {
    let destination = MockServer::start().await;
    let origin = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/sink", destination.uri())),
        )
        .mount(&origin)
        .await;
    let client = quota_client().unwrap();

    let request = client
        .get(format!("{}/quota", origin.uri()))
        .bearer_auth("fixture-secret");
    let result = client.send(request).await;

    assert!(result.is_err());
    assert!(destination.received_requests().await.unwrap().is_empty());
}
