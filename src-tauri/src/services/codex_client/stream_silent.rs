use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::{request, stream::CODEX_IDLE_TIMEOUT_SECS};

pub async fn collect_chat_silent(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    max_output_tokens: Option<u32>,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let resp = if let Some(max) = max_output_tokens {
        request::post_codex_stream_with_timeout(
            model,
            messages,
            tools,
            think,
            reasoning_mode,
            std::time::Duration::from_secs(CODEX_IDLE_TIMEOUT_SECS),
            Some(max),
        )
        .await?
    } else {
        request::post_codex_stream(model, messages, tools, think, reasoning_mode).await?
    };
    consume_sse_silent(
        resp,
        cancel,
        std::time::Duration::from_secs(CODEX_IDLE_TIMEOUT_SECS),
    )
    .await
}

pub async fn collect_chat_silent_for_compression(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    max_output_tokens: Option<u32>,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let timeout = crate::services::compress::timeouts::compression_timeout();
    let resp = request::post_codex_stream_with_timeout(
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        timeout,
        max_output_tokens,
    )
    .await?;
    consume_sse_silent(resp, cancel, timeout).await
}

async fn consume_sse_silent(
    resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: std::time::Duration,
) -> Result<StreamResult, String> {
    let mut sse = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();

    loop {
        let event = tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(idle_timeout) => {
                return Err(format!("Timeout Codex : {}s sans réponse", idle_timeout.as_secs()));
            }
            ev = sse.next() => match ev {
                Some(Ok(e)) => e,
                Some(Err(e)) => return Err(format!("SSE: {e}")),
                None => break,
            },
        };

        if event.data.trim() == "[DONE]" {
            break;
        }
        let parsed: serde_json::Value = match serde_json::from_str(&event.data) {
            Ok(v) => v,
            Err(_) => continue,
        };
        match parsed["type"].as_str().unwrap_or("") {
            "response.reasoning_summary_text.delta" => {
                result
                    .thinking
                    .push_str(parsed["delta"].as_str().unwrap_or(""));
            }
            "response.output_text.delta" => {
                result
                    .content
                    .push_str(parsed["delta"].as_str().unwrap_or(""));
            }
            "response.done" | "response.completed" => {
                if let Some(usage) = parsed.pointer("/response/usage") {
                    result.prompt_tokens = usage["input_tokens"].as_u64().unwrap_or(0) as u32;
                    result.eval_count = usage["output_tokens"].as_u64().unwrap_or(0) as u32;
                }
                break;
            }
            "response.failed" => return Err("Codex: erreur de génération".into()),
            _ => {}
        }
    }

    Ok(result)
}
