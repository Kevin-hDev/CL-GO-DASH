use crate::services::agent_local::types_ollama::ChatMessage;
use serde_json::{json, Value};

pub fn message_to_openai(msg: &ChatMessage, provider_id: &str) -> Value {
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
        "user" => {
            if let Some(images) = &msg.images {
                if !images.is_empty() {
                    let mut parts = vec![json!({"type": "text", "text": msg.content})];
                    for img in images {
                        parts.push(build_image_part(img, provider_id));
                    }
                    return json!({ "role": "user", "content": parts });
                }
            }
            json!({ "role": "user", "content": msg.content })
        }
        _ => {
            json!({ "role": msg.role, "content": msg.content })
        }
    }
}

fn build_image_part(base64_data: &str, provider_id: &str) -> Value {
    let data_url = format!("data:image/png;base64,{base64_data}");
    match provider_id {
        "mistral" => json!({
            "type": "image_url",
            "image_url": data_url,
        }),
        "google" => json!({
            "type": "image_url",
            "image_url": { "url": data_url },
        }),
        _ => json!({
            "type": "image_url",
            "image_url": { "url": data_url },
        }),
    }
}

pub fn messages_to_openai(messages: &[ChatMessage], provider_id: &str) -> Vec<Value> {
    messages.iter().map(|m| message_to_openai(m, provider_id)).collect()
}

pub fn strip_images(messages: &mut [ChatMessage]) {
    for msg in messages.iter_mut() {
        if msg.role == "user" {
            if let Some(images) = &msg.images {
                if !images.is_empty() {
                    let count = images.len();
                    let note = if count == 1 {
                        "\n\n[1 image was attached but this model does not support vision]"
                    } else {
                        "\n\n[Images were attached but this model does not support vision]"
                    };
                    msg.content.push_str(note);
                    msg.images = None;
                }
            }
        }
    }
}
