use std::time::Duration;

use super::convert;
use super::types::{CodexRequest, ReasoningConfig, CODEX_API_BASE};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::codex_oauth::store::CodexTokens;
use crate::services::codex_oauth::token;
use crate::services::secure_http::AuthenticatedClient;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(180);

struct RequestOptions {
    timeout: Duration,
}

pub async fn post_codex_stream(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
) -> Result<reqwest::Response, String> {
    let creds = token::ensure_valid().await?;
    send_request(
        &creds,
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        RequestOptions {
            timeout: REQUEST_TIMEOUT,
        },
    )
    .await
}

pub async fn post_codex_stream_with_timeout(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    timeout: Duration,
) -> Result<reqwest::Response, String> {
    let creds = token::ensure_valid().await?;
    send_request(
        &creds,
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        RequestOptions { timeout },
    )
    .await
}

async fn send_request(
    creds: &CodexTokens,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    _think: bool,
    reasoning_mode: Option<&str>,
    options: RequestOptions,
) -> Result<reqwest::Response, String> {
    let (instructions, input) = convert::convert_messages(messages);
    let converted_tools = convert::convert_tools_to_responses_api(tools);

    let effort = crate::services::reasoning::codex_effort(reasoning_mode);
    let reasoning = Some(ReasoningConfig {
        effort,
        summary: "auto".to_string(),
    });
    let include = Some(vec!["reasoning.encrypted_content".to_string()]);

    let body = CodexRequest {
        model: model.to_string(),
        instructions,
        input,
        stream: true,
        store: false,
        tools: converted_tools,
        tool_choice: if tools.is_empty() {
            None
        } else {
            Some("auto".to_string())
        },
        reasoning,
        include,
    };
    let body_json =
        serde_json::to_string(&body).map_err(|_| "requête Codex invalide".to_string())?;
    let client = AuthenticatedClient::new(options.timeout)
        .map_err(|_| "requête Codex refusée".to_string())?;
    let request = client
        .post(format!("{CODEX_API_BASE}/responses"))
        .bearer_auth(creds.access.as_str())
        .header("chatgpt-account-id", creds.account_hint.as_str())
        .header("OpenAI-Beta", "responses=experimental")
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .body(body_json);
    client
        .send_success(request)
        .await
        .map_err(|_| "requête Codex refusée".to_string())
}
