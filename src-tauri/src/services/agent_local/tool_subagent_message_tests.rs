use super::*;
use crate::services::agent_local::{session_store, subagent_registry, subagent_status};
use serde_json::json;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn duplicate_delivery_returns_explicit_success_without_requeueing() {
    let (parent, mut child) = running_child().await;
    child.messages.push(user_message("Corrige le résultat"));
    session_store::save(&child).await.expect("save delivered prompt");

    let result = message(
        &json!({
            "subagent_id": child.id,
            "prompt": " Corrige   le résultat ",
        }),
        &parent.id,
    )
    .await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    cleanup(&parent.id, &child.id).await;
    assert!(!result.is_error);
    assert!(result.content.contains("déjà"));
    assert!(saved.subagent_queued_prompts.is_empty());
}

#[tokio::test]
async fn message_preserves_original_indentation_and_code_block() {
    let (parent, child) = running_child().await;
    let original = "  vérifie ceci\n\n```rust\n  let x = 1;\n```  ";

    let result = message(
        &json!({ "subagent_id": child.id, "prompt": original }),
        &parent.id,
    )
    .await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    cleanup(&parent.id, &child.id).await;
    assert!(!result.is_error);
    assert_eq!(saved.subagent_queued_prompts, vec![original]);
}

#[tokio::test]
async fn message_rejects_full_queue_without_dropping_entries() {
    let (parent, mut child) = running_child().await;
    child
        .subagent_queued_prompts
        .extend((0..MAX_QUEUED_PROMPTS).map(|index| format!("correction {index}")));
    session_store::save(&child).await.expect("save full queue");

    let result = message(
        &json!({ "subagent_id": child.id, "prompt": "extra" }),
        &parent.id,
    )
    .await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_error);
    assert_eq!(saved.subagent_queued_prompts.len(), MAX_QUEUED_PROMPTS);
}

#[tokio::test]
async fn message_rejects_prompt_over_character_limit() {
    let result = message(
        &json!({
            "subagent_id": uuid::Uuid::new_v4().to_string(),
            "prompt": "x".repeat(MAX_PROMPT_SIZE + 1),
        }),
        "parent",
    )
    .await;

    assert!(result.is_error);
    assert_eq!(result.content, "Instruction sous-agent invalide.");
}

async fn running_child() -> (
    crate::services::agent_local::types_session::AgentSession,
    crate::services::agent_local::types_session::AgentSession,
) {
    let parent = session_store::create_full("Parent", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    let run_id = subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    child.subagent_run_id = Some(run_id);
    session_store::save(&child).await.expect("save run id");
    (parent, child)
}

fn user_message(content: &str) -> crate::services::agent_local::types_session::AgentMessage {
    crate::services::agent_local::types_session::AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".into(),
        content: content.into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
    }
}

async fn cleanup(parent_id: &str, child_id: &str) {
    subagent_registry::unregister(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id)
        .await
        .expect("delete parent");
}
