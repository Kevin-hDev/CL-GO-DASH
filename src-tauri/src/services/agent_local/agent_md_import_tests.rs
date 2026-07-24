use super::*;
use tempfile::TempDir;

fn rule_item(
    source_id: &str,
    source_name: &str,
    path: PathBuf,
) -> crate::services::agent_import::DiscoveredItem {
    crate::services::agent_import::DiscoveredItem {
        public: crate::services::agent_import::ImportItem {
            id: format!("{source_id}:rule:test"),
            name: "rule.md".into(),
            description: String::new(),
            source_id: source_id.into(),
            source_name: source_name.into(),
            kind: crate::services::agent_import::ImportItemKind::Rule,
            selected: true,
            available: true,
            update_available: false,
        },
        path,
        bundle_root: None,
    }
}

#[tokio::test]
async fn imported_claude_and_qwen_documents_are_injected_in_order() {
    let data = TempDir::new().unwrap();
    tokio::fs::write(data.path().join("AGENTS.md"), "Agents rules")
        .await
        .unwrap();
    tokio::fs::write(data.path().join("CLAUDE.md"), "Claude rules")
        .await
        .unwrap();
    tokio::fs::write(data.path().join("QWEN.md"), "Qwen rules")
        .await
        .unwrap();
    let registry = serde_json::json!({
        "version": 1,
        "sources": [
            source_json("claude"),
            source_json("qwen")
        ],
        "documents": [
            document_json("CLAUDE.md", "claude", "0"),
            document_json("QWEN.md", "qwen", "1")
        ]
    });
    tokio::fs::write(
        data.path().join("external-agent-sources.json"),
        serde_json::to_vec(&registry).unwrap(),
    )
    .await
    .unwrap();

    let content = load_agent_md_from(data.path(), None).await.unwrap();

    assert!(content.find("Agents rules").unwrap() < content.find("Claude rules").unwrap());
    assert!(content.find("Claude rules").unwrap() < content.find("Qwen rules").unwrap());
}

#[tokio::test]
async fn disabled_hidden_document_is_not_injected() {
    let data = TempDir::new().unwrap();
    tokio::fs::write(data.path().join("CLAUDE.md"), "Disabled rules")
        .await
        .unwrap();
    let mut document = document_json("CLAUDE.md", "claude", "0");
    document["enabled"] = serde_json::Value::Bool(false);
    let registry = serde_json::json!({
        "version": 1,
        "sources": [source_json("claude")],
        "documents": [document]
    });
    tokio::fs::write(
        data.path().join("external-agent-sources.json"),
        serde_json::to_vec(&registry).unwrap(),
    )
    .await
    .unwrap();

    assert!(load_agent_md_from(data.path(), None).await.is_none());
}

#[tokio::test]
async fn imported_document_stays_injected_when_external_source_is_disabled() {
    let data = TempDir::new().unwrap();
    tokio::fs::write(data.path().join("CLAUDE.md"), "Native imported rules")
        .await
        .unwrap();
    let mut source = source_json("claude");
    source["enabled"] = serde_json::Value::Bool(false);
    let registry = serde_json::json!({
        "version": 1,
        "sources": [source],
        "documents": [document_json("CLAUDE.md", "claude", "0")]
    });
    tokio::fs::write(
        data.path().join("external-agent-sources.json"),
        serde_json::to_vec(&registry).unwrap(),
    )
    .await
    .unwrap();

    let content = load_agent_md_from(data.path(), None).await.unwrap();

    assert!(content.contains("Native imported rules"));
}

fn source_json(source_id: &str) -> serde_json::Value {
    serde_json::json!({
        "sourceId": source_id,
        "enabled": true,
        "skillMode": "none",
        "selectedSkillIds": [],
        "selectedRuleIds": [],
        "selectedDocumentIds": []
    })
}

fn document_json(name: &str, source_id: &str, hash: &str) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "source_id": source_id,
        "source_path": format!("/tmp/{name}"),
        "source_hash": hash.repeat(64),
        "enabled": true
    })
}

#[tokio::test]
async fn external_rules_are_injected_in_stable_source_order() {
    let data = TempDir::new().unwrap();
    let rules = TempDir::new().unwrap();
    let claude = rules.path().join("claude.md");
    let qwen = rules.path().join("qwen.md");
    tokio::fs::write(&claude, "Claude external rule").await.unwrap();
    tokio::fs::write(&qwen, "Qwen external rule").await.unwrap();

    let content = load_agent_md_with_rules(
        data.path(),
        None,
        vec![
            rule_item("qwen", "Qwen Code", qwen),
            rule_item("claude", "Claude Code", claude),
        ],
    )
    .await
    .unwrap();

    assert!(
        content.find("Claude external rule").unwrap()
            < content.find("Qwen external rule").unwrap()
    );
}

#[tokio::test]
async fn oversized_combined_context_adds_explicit_notice() {
    let data = TempDir::new().unwrap();
    tokio::fs::write(data.path().join("AGENTS.md"), "x".repeat(MAX_TOTAL_BYTES))
        .await
        .unwrap();

    let content = load_agent_md_from(data.path(), None).await.unwrap();

    assert!(content.contains(LIMIT_NOTICE));
    assert!(!content.contains(&"x".repeat(1024)));
}
