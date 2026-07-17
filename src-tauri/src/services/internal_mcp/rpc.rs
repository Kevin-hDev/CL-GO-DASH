use super::server::ServerContext;
use serde_json::{json, Value};

pub enum RpcResponse {
    Empty,
    Json(Value),
}

pub async fn handle(context: &ServerContext, body: &[u8]) -> RpcResponse {
    let request: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(_) => return RpcResponse::Json(error(Value::Null, -32700)),
    };
    let id = request.get("id").cloned();
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    if id.is_none() {
        return RpcResponse::Empty;
    }
    let id = id.unwrap_or(Value::Null);
    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "CL-GO", "version": env!("CARGO_PKG_VERSION")},
        }),
        "ping" => json!({}),
        "tools/list" => json!({"tools": super::catalog::definitions(context.provider).await}),
        "tools/call" => {
            let params = &request["params"];
            let Some(name) = params["name"].as_str().filter(|name| name.len() <= 128) else {
                return RpcResponse::Json(error(id, -32602));
            };
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            match super::execute::call(context, name, &arguments).await {
                Ok((content, is_error)) => json!({
                    "content": [{"type":"text", "text":content}],
                    "isError": is_error,
                }),
                Err(_) => return RpcResponse::Json(error(id, -32602)),
            }
        }
        _ => return RpcResponse::Json(error(id, -32601)),
    };
    RpcResponse::Json(json!({"jsonrpc":"2.0", "id":id, "result":result}))
}

fn error(id: Value, code: i32) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "error":{"code":code, "message":"Requête refusée"}})
}
