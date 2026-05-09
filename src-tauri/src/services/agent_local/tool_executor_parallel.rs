use crate::services::agent_local::permission_gate;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_hooks::{run_post_hooks, run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use futures_util::future::join_all;
use serde_json::Value;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

const MAX_PARALLEL: usize = 10;

use super::tool_executor_helpers::{
    check_write_guard, post_record_read, push_tool_result,
};

pub fn is_read_only(name: &str) -> bool {
    matches!(
        name,
        "read_file" | "grep" | "glob" | "list_dir" | "web_search" | "load_skill"
            | "read_spreadsheet" | "read_document" | "read_image"
    )
}

/// Entrée dans le batch : nom, args effectifs, index global.
struct BatchEntry<'a> {
    global_idx: usize,
    name: &'a str,
    effective_args: &'a Value,
}

/// Mode auto : les read-only consécutifs sont parallélisés, les writes sont séquentiels.
/// Si `eager_results` est fourni, les résultats pré-calculés sont réutilisés directement.
pub async fn run_with_parallel_reads(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
    mut eager_results: Option<&mut HashMap<usize, ToolResult>>,
    session_id: &str,
) {
    let mut read_batch: Vec<BatchEntry> = Vec::new();
    let mut indexed_results: Vec<Option<(&str, ToolResult)>> = vec![None; tool_calls.len()];

    let mut i = 0;
    while i <= tool_calls.len() {
        let is_last = i == tool_calls.len();
        let is_write = !is_last && !is_read_only(tool_calls[i].0.as_str());

        if is_last || is_write {
            if !read_batch.is_empty() {
                let batch: Vec<_> = std::mem::take(&mut read_batch);
                flush_read_batch(
                    &batch, &mut indexed_results, working_dir, &cancel,
                    write_guard, &mut eager_results, session_id,
                ).await;
            }

            if is_last {
                break;
            }

            let (name, args) = &tool_calls[i];
            let tr = execute_write(
                on_event, name, args, working_dir, write_guard,
                session_id, cancel.clone(),
            ).await;
            indexed_results[i] = Some((name.as_str(), tr));
            i += 1;
        } else {
            let (name, args) = &tool_calls[i];
            match run_pre_hooks(name, args) {
                PreHookDecision::Deny(msg) => {
                    let tr = tool_dispatcher::enrich_error(ToolResult::err(msg), name);
                    indexed_results[i] = Some((name.as_str(), tr));
                }
                PreHookDecision::Allow => {
                    read_batch.push(BatchEntry {
                        global_idx: i,
                        name: name.as_str(),
                        effective_args: args,
                    });
                }
            }
            i += 1;
        }
    }

    for (idx, slot) in indexed_results.into_iter().enumerate() {
        if let Some((name, tr)) = slot {
            push_tool_result(on_event, messages, name, tr, idx);
        }
    }
}

async fn flush_read_batch<'a>(
    batch: &[BatchEntry<'a>],
    indexed_results: &mut [Option<(&'a str, ToolResult)>],
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
    write_guard: &mut WriteGuard,
    eager_results: &mut Option<&mut HashMap<usize, ToolResult>>,
    session_id: &str,
) {
    for chunk in batch.chunks(MAX_PARALLEL) {
        if cancel.is_cancelled() {
            for entry in chunk {
                indexed_results[entry.global_idx] = Some((entry.name, ToolResult::err("Annulé.")));
            }
            continue;
        }

        let mut pending_indices: Vec<usize> = Vec::new();
        let mut chunk_results: Vec<Option<ToolResult>> = vec![None; chunk.len()];

        for (pos, entry) in chunk.iter().enumerate() {
            if let Some(ref mut eager) = eager_results.as_deref_mut() {
                if let Some(tr) = eager.remove(&entry.global_idx) {
                    post_record_read(entry.name, entry.effective_args, working_dir, &tr, write_guard);
                    let tr = run_post_hooks(entry.name, entry.effective_args, tr);
                    chunk_results[pos] = Some(tr);
                    continue;
                }
            }
            pending_indices.push(pos);
        }

        if !pending_indices.is_empty() {
            let futs: Vec<_> = pending_indices
                .iter()
                .map(|&pos| {
                    let entry = &chunk[pos];
                    tool_dispatcher::dispatch(entry.name, entry.effective_args, working_dir, session_id)
                })
                .collect();
            let dispatched = join_all(futs).await;
            for (pos, tr) in pending_indices.iter().zip(dispatched.into_iter()) {
                let entry = &chunk[*pos];
                post_record_read(entry.name, entry.effective_args, working_dir, &tr, write_guard);
                let tr = run_post_hooks(entry.name, entry.effective_args, tr);
                chunk_results[*pos] = Some(tr);
            }
        }

        for (pos, entry) in chunk.iter().enumerate() {
            let tr = chunk_results[pos].take().unwrap_or_else(|| ToolResult::err("Annulé."));
            indexed_results[entry.global_idx] = Some((entry.name, tr));
        }
    }
}

async fn execute_write(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &Value,
    working_dir: &std::path::Path,
    write_guard: &mut WriteGuard,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    match run_pre_hooks(name, args) {
        PreHookDecision::Deny(msg) => {
            return tool_dispatcher::enrich_error(ToolResult::err(msg), name);
        }
        PreHookDecision::Allow => {}
    }

    if permission_gate::requires_permission(name, args) {
        if !permission_gate::is_allowed(session_id, name).await {
            match permission_gate::request(on_event, name, args, cancel.clone()).await {
                permission_gate::PermissionDecision::Allow => {}
                permission_gate::PermissionDecision::AllowSession => {
                    permission_gate::mark_allowed(session_id, name).await;
                }
                permission_gate::PermissionDecision::Deny => {
                    return ToolResult::err("L'utilisateur a refusé cette action.");
                }
            }
        }
    } else if !permission_gate::check_data_dir_write(
        on_event, name, args, session_id, cancel.clone(),
    ).await {
        return ToolResult::err("L'utilisateur a refusé cette action.");
    }

    let tr = match check_write_guard(name, args, working_dir, write_guard) {
        Err(msg) => tool_dispatcher::enrich_error(ToolResult::err(msg), name),
        Ok(()) => {
            if cancel.is_cancelled() {
                ToolResult::err("Annulé.")
            } else {
                tool_dispatcher::dispatch(name, args, working_dir, session_id).await
            }
        }
    };
    run_post_hooks(name, args, tr)
}
