use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use futures_util::future::join_all;
use tokio_util::sync::CancellationToken;

use super::tool_executor_helpers::{
    check_write_guard, post_record_read, push_tool_result,
};

pub fn is_read_only(name: &str) -> bool {
    matches!(
        name,
        "read_file" | "grep" | "glob" | "list_dir" | "web_search" | "web_fetch" | "load_skill"
    )
}

/// Mode auto : les read-only consécutifs sont parallélisés, les writes sont séquentiels.
pub async fn run_with_parallel_reads(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
) {
    let mut read_batch: Vec<(usize, &str, &serde_json::Value)> = Vec::new();
    let mut ordered_results: Vec<(&str, ToolResult)> = Vec::new();

    let mut i = 0;
    while i <= tool_calls.len() {
        let is_last = i == tool_calls.len();
        let is_write = !is_last && !is_read_only(tool_calls[i].0.as_str());

        if is_last || is_write {
            // Flush le batch read-only accumulé
            if !read_batch.is_empty() {
                let batch: Vec<_> = std::mem::take(&mut read_batch);
                let futs: Vec<_> = batch
                    .iter()
                    .map(|(_, name, args)| tool_dispatcher::dispatch(name, args, working_dir))
                    .collect();
                let results = join_all(futs).await;
                for ((_, name, args), tr) in batch.iter().zip(results.into_iter()) {
                    post_record_read(name, args, working_dir, &tr, write_guard);
                    ordered_results.push((name, tr));
                }
            }

            if is_last {
                break;
            }

            // Exécute le write séquentiellement
            let (name, args) = &tool_calls[i];
            let tr = match check_write_guard(name, args, working_dir, write_guard) {
                Err(msg) => ToolResult::err(msg),
                Ok(()) => {
                    if cancel.is_cancelled() {
                        ToolResult::err("Annulé.")
                    } else {
                        tool_dispatcher::dispatch(name, args, working_dir).await
                    }
                }
            };
            ordered_results.push((name.as_str(), tr));
            i += 1;
        } else {
            // Accumule le read-only dans le batch
            let (name, args) = &tool_calls[i];
            read_batch.push((i, name.as_str(), args));
            i += 1;
        }
    }

    for (name, tr) in ordered_results {
        push_tool_result(on_event, messages, name, tr);
    }
}
