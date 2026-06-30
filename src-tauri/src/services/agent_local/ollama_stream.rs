use crate::services::agent_local::ollama_base_url;
use crate::services::agent_local::ollama_stream_process::{flush_filter, process_chunk};
use crate::services::agent_local::ollama_stream_retry::build_retry_request;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{
    ChatRequest, StreamEvent, StreamOutcome, StreamResult,
};
use crate::services::compress::realtime_budget::RealtimeBudget;
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
    realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, String> {
    stream_chat_inner(
        on_event,
        request,
        cancel,
        false,
        Some(tool_tx),
        buffer_content,
        realtime_budget,
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
    mut realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, String> {
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
                realtime_budget,
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
        .map(|r| r.map_err(std::io::Error::other));
    let mut lines = BufReader::new(StreamReader::new(byte_stream)).lines();

    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;
    let mut result = StreamResult::default();
    let mut think_filter = ThinkTagFilter::new();
    let mut interrupted = false;

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
                        if should_interrupt(&mut realtime_budget, token_count, !result.tool_calls.is_empty()) {
                            interrupted = true;
                            break;
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
    if interrupted {
        flush_filter(
            &mut think_filter,
            on_event,
            &mut token_count,
            &mut first_token,
            &mut result,
            buffer_content,
        );
        Ok(StreamOutcome::InterruptedForCompression(result))
    } else {
        Ok(StreamOutcome::Completed(result))
    }
}

fn should_interrupt(
    budget: &mut Option<RealtimeBudget>,
    token_count: u32,
    has_tool_call: bool,
) -> bool {
    !has_tool_call
        && budget
            .as_mut()
            .is_some_and(|budget| budget.should_interrupt(token_count))
}
