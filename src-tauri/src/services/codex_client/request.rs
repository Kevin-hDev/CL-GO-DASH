use std::sync::Mutex;
use std::time::Duration;

use super::convert;
use super::types::{self, CodexRequest, CODEX_API_BASE};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::codex_oauth::store::CodexTokens;
use crate::services::codex_oauth::token;

static EFFORT: Mutex<String> = Mutex::new(String::new());

pub fn set_effort(level: &str) {
    if types::CODEX_EFFORT_LEVELS.contains(&level) {
        if let Ok(mut val) = EFFORT.lock() {
            *val = level.to_string();
        }
    }
}

pub fn get_effort() -> String {
    EFFORT.lock().ok()
        .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) })
        .unwrap_or_else(|| "medium".to_string())
}

const REQUEST_TIMEOUT: Duration = Duration::from_secs(180);

pub async fn post_codex_stream(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
) -> Result<reqwest::Response, String> {
    let creds = token::ensure_valid().await?;
    send_request(&creds, model, messages, tools, think).await
}

async fn send_request(
    creds: &CodexTokens,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    _think: bool,
) -> Result<reqwest::Response, String> {
    let (instructions, input) = convert::convert_messages(messages);
    let converted_tools = convert::convert_tools_to_responses_api(tools);

    let effort = get_effort();
    let reasoning = Some(types::ReasoningConfig {
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
        tool_choice: if tools.is_empty() { None } else { Some("auto".to_string()) },
        reasoning,
        include,
    };
    let url = format!("{CODEX_API_BASE}/responses");
    let body_json = serde_json::to_string(&body).map_err(|e| format!("json: {e}"))?;

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", creds.access.as_str()))
        .header("chatgpt-account-id", &creds.account_id)
        .header("OpenAI-Beta", "responses=experimental")
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .body(body_json)
        .send()
        .await
        .map_err(|e| format!("codex request: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let safe = &text[..text.len().min(500)];
        return Err(format!("Codex API {status}: {safe}"));
    }
    Ok(resp)
}
