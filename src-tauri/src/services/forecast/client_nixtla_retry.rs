use reqwest::{Client, Response, StatusCode};
use serde_json::Value;
use std::time::Duration;
use zeroize::Zeroizing;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const RETRY_DELAYS_SECS: [u64; 5] = [2, 4, 8, 16, 32];

pub async fn post_json_with_retry(
    client: &Client,
    endpoint: &str,
    api_key: &Zeroizing<String>,
    payload: &Value,
) -> Result<Response, String> {
    for attempt in 0..=RETRY_DELAYS_SECS.len() {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(RETRY_DELAYS_SECS[attempt - 1])).await;
        }

        match send_once(client, endpoint, api_key, payload).await {
            Ok(resp) if resp.status().is_success() => return Ok(resp),
            Ok(resp) if should_retry_status(resp.status()) && attempt < RETRY_DELAYS_SECS.len() => {
            }
            Ok(_) => return Err("Erreur du service de prédiction".to_string()),
            Err(error) if should_retry_error(&error) && attempt < RETRY_DELAYS_SECS.len() => {}
            Err(_) => return Err("Erreur du service de prédiction".to_string()),
        }
    }

    Err("Erreur du service de prédiction".to_string())
}

async fn send_once(
    client: &Client,
    endpoint: &str,
    api_key: &Zeroizing<String>,
    payload: &Value,
) -> Result<Response, reqwest::Error> {
    client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key.as_str()))
        .header("Content-Type", "application/json")
        .json(payload)
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await
}

fn should_retry_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

fn should_retry_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect()
}
