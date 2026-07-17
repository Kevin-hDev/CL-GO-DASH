use serde_json::Value;
use tokio::io::{AsyncRead, AsyncReadExt};

const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

pub struct JsonLineReader<R> {
    inner: R,
    buffer: Vec<u8>,
}

impl<R: AsyncRead + Unpin> JsonLineReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: Vec::with_capacity(8192),
        }
    }

    pub async fn next_value(&mut self) -> Result<Value, String> {
        loop {
            if let Some(position) = self.buffer.iter().position(|byte| *byte == b'\n') {
                let mut line: Vec<u8> = self.buffer.drain(..=position).collect();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                if line.is_empty() {
                    continue;
                }
                return serde_json::from_slice(&line)
                    .map_err(|_| "Message ACP invalide".to_string());
            }
            if self.buffer.len() >= MAX_MESSAGE_BYTES {
                self.buffer.clear();
                return Err("Message ACP trop grand".to_string());
            }
            let mut chunk = [0u8; 8192];
            let count = self
                .inner
                .read(&mut chunk)
                .await
                .map_err(|_| "Lecture ACP impossible".to_string())?;
            if count == 0 {
                return Err("Client ACP fermé".to_string());
            }
            if self.buffer.len().saturating_add(count) > MAX_MESSAGE_BYTES {
                self.buffer.clear();
                return Err("Message ACP trop grand".to_string());
            }
            self.buffer.extend_from_slice(&chunk[..count]);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AcpUpdate {
    Text(String),
    Thought(String),
    ToolCall {
        id: String,
        name: String,
        arguments: Value,
        source: String,
        kind: Option<String>,
    },
    ToolUpdate {
        id: String,
        status: Option<String>,
        content: Option<String>,
    },
    Plan(Value),
    Unknown(String),
}

impl AcpUpdate {
    pub fn from_message(message: &Value) -> Self {
        let update = &message["params"]["update"];
        match update["sessionUpdate"].as_str() {
            Some("agent_message_chunk") => text_update(update, false),
            Some("agent_thought_chunk") => text_update(update, true),
            Some("tool_call") => Self::ToolCall {
                id: update["toolCallId"]
                    .as_str()
                    .unwrap_or("")
                    .chars()
                    .take(128)
                    .collect(),
                name: tool_name(update),
                arguments: update.get("rawInput").cloned().unwrap_or(Value::Null),
                source: tool_source(update),
                kind: update["kind"]
                    .as_str()
                    .map(|value| value.chars().take(32).collect()),
            },
            Some("tool_call_update") => Self::ToolUpdate {
                id: update["toolCallId"]
                    .as_str()
                    .unwrap_or("")
                    .chars()
                    .take(128)
                    .collect(),
                status: update["status"]
                    .as_str()
                    .map(|value| value.chars().take(32).collect()),
                content: tool_content(update),
            },
            Some("plan") => Self::Plan(update.clone()),
            _ => Self::Unknown(
                update["sessionUpdate"]
                    .as_str()
                    .unwrap_or("unknown")
                    .chars()
                    .take(64)
                    .collect(),
            ),
        }
    }
}

fn text_update(update: &Value, thought: bool) -> AcpUpdate {
    let text: String = update["content"]["text"]
        .as_str()
        .unwrap_or("")
        .chars()
        .take(MAX_MESSAGE_BYTES)
        .collect();
    if thought {
        AcpUpdate::Thought(text)
    } else {
        AcpUpdate::Text(text)
    }
}

fn tool_name(update: &Value) -> String {
    let kind = update["kind"].as_str().unwrap_or("");
    let value = if kind.eq_ignore_ascii_case("execute") {
        "bash"
    } else {
        update["title"]
            .as_str()
            .filter(|value| !value.is_empty())
            .unwrap_or(kind)
    };
    if value.is_empty() { "tool" } else { value }
        .chars()
        .take(128)
        .collect()
}

fn tool_source(update: &Value) -> String {
    let title = update["title"].as_str().unwrap_or("");
    let is_mcp = title != "Bash"
        && (title.eq_ignore_ascii_case("bash")
            || title.to_ascii_lowercase().contains("cl-go")
            || [
                "search_mcp_tools",
                "forecast",
                "forecast_models",
                "forecast_analyze",
                "forecast_read",
                "read_spreadsheet",
                "read_document",
                "read_image",
                "write_spreadsheet",
                "write_document",
                "process_image",
            ]
            .contains(&title));
    if is_mcp { "mcp" } else { "native" }.to_string()
}

fn tool_content(update: &Value) -> Option<String> {
    let items = update["content"].as_array()?;
    let mut output = String::new();
    for item in items.iter().take(32) {
        let text = item["content"]["text"]
            .as_str()
            .or_else(|| item["text"].as_str());
        if let Some(text) = text {
            if output.len() >= MAX_MESSAGE_BYTES {
                break;
            }
            output.extend(text.chars().take(MAX_MESSAGE_BYTES - output.len()));
        }
    }
    (!output.is_empty()).then_some(output)
}
