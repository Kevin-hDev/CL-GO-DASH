use super::{
    stream_chunk::{self, ParsedChunk},
    stream_sse::is_done_marker,
    stream_tools::ToolCallAccumulator,
};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamResult};
use crate::services::stream_utils::{compute_tps, FilteredChunk, ThinkTagFilter};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

pub(super) async fn consume_stream(
    on_event: &AgentEventEmitter,
    resp: reqwest::Response,
    cancel: CancellationToken,
) -> Result<(StreamResult, u32, std::time::Instant), String> {
    let mut stream = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;
    let mut acc = ToolCallAccumulator::new();
    let mut think_filter = ThinkTagFilter::new();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(super::timeouts::idle_timeout()) => {
                return Err("Timeout : aucune réponse du modèle depuis 180s".to_string());
            }
            event = stream.next() => {
                let Some(event) = event else { break; };
                let event = event.map_err(|e| format!("SSE: {e}"))?;
                if is_done_marker(&event.data) { break; }
                process_chunk(&event.data, on_event, &mut token_count, &mut first_token, &mut result, &mut acc, &mut think_filter);
            }
        }
    }

    for chunk in think_filter.flush() {
        match chunk {
            FilteredChunk::Thinking(t) => {
                result.thinking.push_str(&t);
                let _ = on_event.send(StreamEvent::Thinking { content: t });
            }
            FilteredChunk::Content(c) => {
                result.content.push_str(&c);
                token_count += 1;
                let tps = compute_tps(token_count, first_token);
                let _ = on_event.send(StreamEvent::Token {
                    content: c,
                    token_count,
                    tps,
                });
            }
        }
    }

    let (tool_calls, ids, extra_content) = acc.finalize();
    for (i, (name, args)) in tool_calls.iter().enumerate() {
        let _ = on_event.send(StreamEvent::ToolCall {
            name: name.clone(),
            arguments: args.clone(),
        });
        result.tool_calls.push((name.clone(), args.clone()));
        if let Some(id) = ids.get(i) {
            result.tool_call_ids.push(id.clone());
        }
        result
            .tool_call_extra_content
            .push(extra_content.get(i).cloned().flatten());
    }

    let first = first_token.unwrap_or_else(std::time::Instant::now);
    Ok((result, token_count, first))
}

fn process_chunk(
    data: &str,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
    result: &mut StreamResult,
    acc: &mut ToolCallAccumulator,
    think_filter: &mut ThinkTagFilter,
) {
    for chunk in stream_chunk::parse(data) {
        match chunk {
            ParsedChunk::Thinking(thinking) => {
                result.thinking.push_str(&thinking);
                *token_count += 1;
                let _ = on_event.send(StreamEvent::Thinking { content: thinking });
            }
            ParsedChunk::Content(content) => {
                for filtered in think_filter.feed(&content) {
                    match filtered {
                        FilteredChunk::Thinking(t) => {
                            result.thinking.push_str(&t);
                            let _ = on_event.send(StreamEvent::Thinking { content: t });
                        }
                        FilteredChunk::Content(c) => {
                            result.content.push_str(&c);
                            *token_count += 1;
                            if first_token.is_none() {
                                *first_token = Some(std::time::Instant::now());
                            }
                            let tps = compute_tps(*token_count, *first_token);
                            let _ = on_event.send(StreamEvent::Token {
                                content: c,
                                token_count: *token_count,
                                tps,
                            });
                        }
                    }
                }
            }
            ParsedChunk::ToolCalls(tool_calls) => acc.ingest(&tool_calls),
            ParsedChunk::Usage {
                completion_tokens,
                prompt_tokens,
            } => {
                result.eval_count = completion_tokens;
                result.prompt_tokens = prompt_tokens;
            }
        }
    }
}
