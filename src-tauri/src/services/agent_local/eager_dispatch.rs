use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_executor_parallel::is_read_only;
use crate::services::agent_local::types_tools::ToolResult;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;

const MAX_EAGER: usize = 10;

/// Écoute les tool calls en provenance du stream Ollama et dispatch les read-only immédiatement,
/// AVANT que le stream soit terminé.
///
/// Retourne un HashMap<usize, ToolResult> indexé par position dans l'ordre de réception.
/// Les tools non-eagerable sont ignorés (ils seront dispatchés normalement après le stream).
pub async fn collect_eager_results(
    mut rx: mpsc::UnboundedReceiver<(usize, String, serde_json::Value)>,
    working_dir: PathBuf,
    session_id: String,
) -> HashMap<usize, ToolResult> {
    let mut handles: Vec<tokio::task::JoinHandle<(usize, ToolResult)>> = Vec::new();
    let mut count = 0;

    while let Some((idx, name, args)) = rx.recv().await {
        if !is_read_only(&name) || count >= MAX_EAGER {
            continue;
        }
        count += 1;
        let wd = working_dir.clone();
        let sid = session_id.clone();
        let handle = tokio::spawn(async move {
            let result = tool_dispatcher::dispatch(&name, &args, &wd, &sid).await;
            (idx, result)
        });
        handles.push(handle);
    }

    let mut results = HashMap::new();
    for handle in handles {
        if let Ok((idx, result)) = handle.await {
            results.insert(idx, result);
        }
    }
    results
}
