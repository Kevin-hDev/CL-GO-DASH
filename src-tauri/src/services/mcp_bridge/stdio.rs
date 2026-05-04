use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use super::process_manager::{self, ProcessHandle};
use super::transport::{next_id, sanitize_tools, McpToolDef, McpTransport};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(360);
const MAX_LINE_BYTES: usize = 1_048_576;

pub struct StdioTransport {
    pub connector_id: String,
    pub install_command: String,
    pub env_keys: Vec<(String, String)>,
}

impl StdioTransport {
    pub fn new(
        connector_id: String,
        install_command: String,
        env_keys: Vec<(String, String)>,
    ) -> Self {
        Self {
            connector_id,
            install_command,
            env_keys,
        }
    }

    async fn ensure_running(&self) -> Result<ProcessHandle, String> {
        if process_manager::is_alive(&self.connector_id) {
            if let Some(handle) = process_manager::get_handle(&self.connector_id) {
                process_manager::touch(&self.connector_id);
                return Ok(handle);
            }
        }

        process_manager::shutdown_one(&self.connector_id);
        let handle = process_manager::spawn(
            &self.connector_id,
            &self.install_command,
            &self.env_keys,
        )?;
        self.handshake(&handle).await?;
        Ok(handle)
    }

    async fn handshake(&self, handle: &ProcessHandle) -> Result<(), String> {
        let init = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "id": next_id(),
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {
                    "name": "CL-GO-DASH",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        let _ = self.send(handle, &init).await?;

        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        self.write_line(handle, &notif).await?;
        Ok(())
    }

    async fn send(&self, handle: &ProcessHandle, request: &Value) -> Result<Value, String> {
        self.write_line(handle, request).await?;
        self.read_response(handle).await
    }

    async fn write_line(&self, handle: &ProcessHandle, msg: &Value) -> Result<(), String> {
        let mut line =
            serde_json::to_string(msg).map_err(|_| "sérialisation échouée")?;
        line.push('\n');

        let mut stdin = handle.stdin.lock().await;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|_| "impossible d'écrire sur stdin du process MCP")?;
        stdin
            .flush()
            .await
            .map_err(|_| "flush stdin échoué")?;
        Ok(())
    }

    async fn read_response(&self, handle: &ProcessHandle) -> Result<Value, String> {
        let mut reader = handle.reader.lock().await;
        let mut line = String::new();

        let read_result =
            tokio::time::timeout(REQUEST_TIMEOUT, reader.read_line(&mut line)).await;

        match read_result {
            Err(_) => Err("timeout : le serveur MCP n'a pas répondu".to_string()),
            Ok(Err(_)) => Err("erreur de lecture stdout du process MCP".to_string()),
            Ok(Ok(0)) => Err("le process MCP s'est arrêté".to_string()),
            Ok(Ok(n)) if n > MAX_LINE_BYTES => {
                Err("réponse MCP trop volumineuse".to_string())
            }
            Ok(Ok(_)) => {
                let trimmed = line.trim();
                serde_json::from_str(trimmed)
                    .map_err(|_| "réponse JSON-RPC invalide".to_string())
            }
        }
    }

    fn extract_call_result(&self, resp: &Value) -> Result<String, String> {
        if let Some(err) = resp.get("error") {
            let msg = err["message"].as_str().unwrap_or("erreur inconnue");
            return Err(format!("erreur MCP : {msg}"));
        }

        let result = resp.get("result").ok_or("réponse vide du serveur MCP")?;

        if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
            let texts: Vec<&str> = content
                .iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                .collect();
            if !texts.is_empty() {
                return Ok(texts.join("\n"));
            }
        }

        Ok(serde_json::to_string_pretty(result).unwrap_or_default())
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn list_tools(&self) -> Result<Vec<McpToolDef>, String> {
        let handle = self.ensure_running().await?;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": next_id()
        });

        let resp = self.send(&handle, &body).await?;

        let tools_val = resp
            .get("result")
            .and_then(|r| r.get("tools").cloned())
            .ok_or("réponse tools/list invalide")?;

        let tools: Vec<McpToolDef> =
            serde_json::from_value(tools_val).map_err(|_| "format tools invalide")?;

        Ok(sanitize_tools(tools))
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<String, String> {
        let handle = self.ensure_running().await?;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "id": next_id(),
            "params": { "name": name, "arguments": args }
        });

        let resp = self.send(&handle, &body).await?;
        self.extract_call_result(&resp)
    }

    fn transport_type(&self) -> &'static str {
        "stdio"
    }
}
