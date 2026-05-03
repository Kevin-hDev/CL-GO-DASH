use std::time::Duration;

use super::types::{CodexRequest, CODEX_API_BASE};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::codex_oauth::store::CodexTokens;
use crate::services::codex_oauth::token;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(180);

pub fn convert_messages(messages: &[ChatMessage]) -> (String, Vec<serde_json::Value>) {
    let mut instructions = String::new();
    let mut input = Vec::new();

    for msg in messages {
        if msg.role == "system" {
            if !instructions.is_empty() {
                instructions.push_str("\n\n");
            }
            instructions.push_str(&msg.content);
            continue;
        }
        let mut obj = serde_json::json!({
            "role": msg.role,
            "content": msg.content,
        });
        if let Some(ref tc) = msg.tool_calls {
            obj["tool_calls"] = serde_json::to_value(tc).unwrap_or_default();
        }
        if let Some(ref id) = msg.tool_call_id {
            obj["tool_call_id"] = serde_json::Value::String(id.clone());
        }
        input.push(obj);
    }
    (instructions, input)
}

pub async fn post_codex_stream(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
) -> Result<reqwest::Response, String> {
    let creds = token::ensure_valid().await?;
    send_request(&creds, model, messages, tools).await
}

async fn send_request(
    creds: &CodexTokens,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
) -> Result<reqwest::Response, String> {
    let (instructions, input) = convert_messages(messages);
    let body = CodexRequest {
        model: model.to_string(),
        instructions,
        input,
        stream: true,
        store: false,
        tools: tools.to_vec(),
        tool_choice: if tools.is_empty() { None } else { Some("auto".to_string()) },
        temperature: Some(0.7),
    };
    let url = format!("{CODEX_API_BASE}/responses");
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
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("codex request: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let safe = &text[..text.len().min(200)];
        return Err(format!("Codex API {status}: {safe}"));
    }
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_extracts_system_as_instructions() {
        let msgs = vec![
            ChatMessage {
                role: "system".into(),
                content: "Tu es un assistant.".into(),
                ..Default::default()
            },
            ChatMessage {
                role: "user".into(),
                content: "Bonjour".into(),
                ..Default::default()
            },
        ];
        let (instructions, input) = convert_messages(&msgs);
        assert_eq!(instructions, "Tu es un assistant.");
        assert_eq!(input.len(), 1);
        assert_eq!(input[0]["role"], "user");
        assert_eq!(input[0]["content"], "Bonjour");
    }

    #[test]
    fn convert_handles_no_system() {
        let msgs = vec![ChatMessage {
            role: "user".into(),
            content: "Hello".into(),
            ..Default::default()
        }];
        let (instructions, input) = convert_messages(&msgs);
        assert!(instructions.is_empty());
        assert_eq!(input.len(), 1);
    }
}
