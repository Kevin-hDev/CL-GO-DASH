use super::JsonLineReader;
use serde_json::{json, Value};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

const MAX_OUTBOUND_BYTES: usize = 1024 * 1024;
const MAX_SKIPPED_MESSAGES: usize = 256;

pub struct AcpConnection<R, W> {
    reader: JsonLineReader<R>,
    writer: W,
}

impl<R, W> AcpConnection<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: JsonLineReader::new(reader),
            writer,
        }
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.write_value(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }))
        .await?;
        for _ in 0..MAX_SKIPPED_MESSAGES {
            let message = self.reader.next_value().await?;
            if message["id"].as_str() == Some(id.as_str()) && message.get("method").is_none() {
                if message.get("error").is_some() {
                    return Err("Requête ACP refusée".to_string());
                }
                return Ok(message.get("result").cloned().unwrap_or(Value::Null));
            }
            if message.get("id").is_some() && message.get("method").is_some() {
                self.reject_request(&message).await?;
            }
        }
        Err("Trop de messages ACP".to_string())
    }

    pub async fn start_request(&mut self, method: &str, params: Value) -> Result<String, String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.write_value(&json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
            .await?;
        Ok(id)
    }

    pub async fn next_message(&mut self) -> Result<Value, String> {
        self.reader.next_value().await
    }

    pub async fn respond(&mut self, id: &Value, result: Value) -> Result<(), String> {
        self.write_value(&json!({"jsonrpc":"2.0","id":id,"result":result}))
            .await
    }

    pub async fn respond_error(&mut self, id: &Value) -> Result<(), String> {
        self.write_value(&json!({
            "jsonrpc":"2.0",
            "id":id,
            "error":{"code":-32601,"message":"Method not supported"},
        }))
        .await
    }

    pub async fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.write_value(&json!({"jsonrpc":"2.0","method":method,"params":params}))
            .await
    }

    async fn reject_request(&mut self, message: &Value) -> Result<(), String> {
        self.write_value(&json!({
            "jsonrpc": "2.0",
            "id": message["id"],
            "error": {"code": -32601, "message": "Method not supported"},
        }))
        .await
    }

    async fn write_value(&mut self, value: &Value) -> Result<(), String> {
        let mut data = zeroize::Zeroizing::new(
            serde_json::to_vec(value).map_err(|_| "Message ACP invalide".to_string())?,
        );
        if data.len() > MAX_OUTBOUND_BYTES {
            return Err("Message ACP trop grand".to_string());
        }
        data.push(b'\n');
        self.writer
            .write_all(&data)
            .await
            .map_err(|_| "Écriture ACP impossible".to_string())?;
        self.writer
            .flush()
            .await
            .map_err(|_| "Écriture ACP impossible".to_string())
    }
}
