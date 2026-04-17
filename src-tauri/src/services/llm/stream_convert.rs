//! Conversion des messages format Ollama (interne) vers payload OpenAI-compat.
//!
//! Différences clés :
//! - `role: "tool"` OpenAI requiert `tool_call_id` (pas `tool_name`)
//! - `tool_calls[].function.arguments` OpenAI est un **string JSON**, pas un objet
//! - `tool_calls[].id` requis, `type: "function"` requis

use crate::services::agent_local::types_ollama::ChatMessage;
use serde_json::{json, Value};

/// Convertit un `ChatMessage` (Ollama) vers le format de message OpenAI-compat.
pub fn message_to_openai(msg: &ChatMessage) -> Value {
    match msg.role.as_str() {
        "tool" => {
            let mut obj = json!({
                "role": "tool",
                "content": msg.content,
            });
            if let Some(id) = &msg.tool_call_id {
                obj["tool_call_id"] = json!(id);
            }
            obj
        }
        "assistant" => {
            let content = if msg.content.is_empty() && msg.tool_calls.is_some() {
                Value::Null
            } else {
                json!(msg.content)
            };
            let mut obj = json!({
                "role": "assistant",
                "content": content,
            });
            if let Some(tcs) = &msg.tool_calls {
                let tc_arr: Vec<Value> = tcs
                    .iter()
                    .enumerate()
                    .map(|(i, tc)| {
                        let args_str = serde_json::to_string(&tc.function.arguments)
                            .unwrap_or_else(|_| "{}".to_string());
                        let id = tc
                            .id
                            .clone()
                            .unwrap_or_else(|| format!("call_{}", i));
                        json!({
                            "id": id,
                            "type": "function",
                            "function": {
                                "name": tc.function.name,
                                "arguments": args_str,
                            }
                        })
                    })
                    .collect();
                obj["tool_calls"] = json!(tc_arr);
            }
            obj
        }
        _ => {
            // system, user — content toujours string simple.
            // Les images Ollama ne sont pas converties (format incompatible).
            // Le support vision multimodal pour API sera ajouté séparément.
            json!({ "role": msg.role, "content": msg.content })
        }
    }
}

/// Convertit un batch de messages vers un array JSON OpenAI-compat.
pub fn messages_to_openai(messages: &[ChatMessage]) -> Vec<Value> {
    messages.iter().map(message_to_openai).collect()
}
