use crate::services::agent_local::ollama_base_url;
use crate::services::agent_local::ollama_stream_process::process_chunk;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatRequest, StreamEvent, StreamResult};
use crate::services::llm::vision;
use crate::services::stream_utils::ThinkTagFilter;
use futures_util::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio_util::io::StreamReader;
use tokio_util::sync::CancellationToken;

pub use crate::services::agent_local::ollama_collect::{
    collect_chat, collect_chat_with_timeout_and_limit,
};

/// Variante avec eager dispatch : les tool calls sont envoyés via `tool_tx` dès réception.
pub async fn stream_chat_with_tool_notify(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: CancellationToken,
    tool_tx: mpsc::UnboundedSender<(usize, String, serde_json::Value)>,
    buffer_content: bool,
) -> Result<StreamResult, String> {
    stream_chat_inner(
        on_event,
        request,
        cancel,
        false,
        Some(tool_tx),
        buffer_content,
    )
    .await
}

async fn stream_chat_inner(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: CancellationToken,
    emit_done: bool,
    tool_tx: Option<mpsc::UnboundedSender<(usize, String, serde_json::Value)>>,
    buffer_content: bool,
) -> Result<StreamResult, String> {
    let client = reqwest::Client::new();
    let resp = match client
        .post(format!("{}/api/chat", ollama_base_url()))
        .json(request)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let msg = if e.is_connect() || e.is_timeout() {
                "ollama_connection_lost".to_string()
            } else {
                format!("Ollama: {e}")
            };
            let _ = on_event.send(StreamEvent::Error {
                message: msg.clone(),
                is_connection: e.is_connect() || e.is_timeout(),
                diagnostic: None,
            });
            return Err(msg);
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if let Some(retry_req) = build_retry_request(request, &body) {
            let feature = if retry_req.think != request.think {
                "thinking"
            } else if retry_req.tools != request.tools {
                "tools"
            } else {
                "images"
            };
            eprintln!("[ollama-stream] modèle sans {feature}, retry");
            if feature == "images" {
                let _ = on_event.send(StreamEvent::Notice {
                    message_key: vision::NOTICE_UNSUPPORTED_MODEL.to_string(),
                });
            }
            return Box::pin(stream_chat_inner(
                on_event,
                &retry_req,
                cancel,
                emit_done,
                tool_tx,
                buffer_content,
            ))
            .await;
        }
        eprintln!("[ollama-stream] HTTP {status}: {body}");
        let msg = format!("Ollama HTTP {status}");
        let _ = on_event.send(StreamEvent::Error {
            message: "Erreur serveur Ollama".into(),
            is_connection: false,
            diagnostic: None,
        });
        return Err(msg);
    }

    let byte_stream = resp
        .bytes_stream()
        .map(|r| r.map_err(|e| std::io::Error::other(e)));
    let mut lines = BufReader::new(StreamReader::new(byte_stream)).lines();

    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;
    let mut result = StreamResult::default();
    let mut think_filter = ThinkTagFilter::new();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                return Err("Annulé".to_string());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
                let msg = "Timeout : aucune réponse d'Ollama depuis 5 min".to_string();
                let _ = on_event.send(StreamEvent::Error { message: msg.clone(), is_connection: false, diagnostic: None });
                return Err(msg);
            }
            line = lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if let Err(e) = process_chunk(
                            &text, on_event, &mut token_count, &mut first_token,
                            &mut result, emit_done, tool_tx.as_ref(), &mut think_filter,
                            buffer_content,
                        ) {
                            let _ = on_event.send(StreamEvent::Error { message: e.clone(), is_connection: false, diagnostic: None });
                            return Err(e);
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let is_conn = e.kind() == std::io::ErrorKind::ConnectionReset
                            || e.kind() == std::io::ErrorKind::ConnectionAborted
                            || e.kind() == std::io::ErrorKind::UnexpectedEof
                            || e.to_string().contains("decoding");
                        let msg = "ollama_connection_lost".to_string();
                        let _ = on_event.send(StreamEvent::Error { message: msg.clone(), is_connection: is_conn, diagnostic: None });
                        return Err(msg);
                    }
                }
            }
        }
    }
    Ok(result)
}

fn build_retry_request(request: &ChatRequest, error_body: &str) -> Option<ChatRequest> {
    let mut retry = request.clone();
    let mut changed = false;
    if error_body.contains("does not support thinking")
        && request.think.as_ref().is_some_and(|think| think.enabled())
    {
        retry.think = Some(crate::services::agent_local::types_ollama::OllamaThink::Bool(false));
        changed = true;
    }
    if error_body.contains("does not support tools") && request.tools.is_some() {
        retry.tools = None;
        changed = true;
    }
    if error_body.contains("does not support images") {
        for msg in &mut retry.messages {
            msg.images = None;
        }
        changed = true;
    }
    if changed {
        Some(retry)
    } else {
        None
    }
}
