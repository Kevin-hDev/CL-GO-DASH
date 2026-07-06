use super::*;
use crate::services::agent_local::types_session::{AgentSession, CloneMode};
use chrono::Utc;
use git2::{Repository, Signature};
use uuid::Uuid;

fn init_repo_with_commit() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    std::fs::write(tmp.path().join("README.md"), "init").expect("write file");
    let mut index = repo.index().expect("index");
    index
        .add_path(std::path::Path::new("README.md"))
        .expect("add");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    let sig = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("commit");
    tmp
}

fn session(id: &str, parent: Option<&str>, git_branch: Option<&str>) -> AgentSession {
    AgentSession {
        id: id.into(),
        name: id.into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        accumulated_tokens: 0,
        messages: vec![],
        todos: vec![],
        todo_neglect_count: 0,
        todo_runs: vec![],
        active_todo_run_id: None,
        stream_failures: vec![],
        diagnostic_runs: vec![],
        plan_mode_enabled: false,
        plan_runs: vec![],
        active_plan_id: None,
        plan_workflow_status: Default::default(),
        plan_approval_decision: None,
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        working_dir: String::new(),
        parent_session_id: None,
        subagent_type: None,
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: None,
        subagent_run_id: None,
        clone_parent_session_id: parent.map(str::to_string),
        clone_parent_message_id: parent.map(|_| "m1".into()),
        clone_mode: parent.map(|_| CloneMode::Cut),
        clone_summary: None,
        clone_read_files: vec![],
        clone_modified_files: vec![],
        clone_root_session_id: parent.map(str::to_string),
        git_branch: git_branch.map(str::to_string),
    }
}

#[test]
fn branch_name_shape_is_valid() {
    let name = random_branch_name();
    assert!(name.starts_with("clone-"));
    assert_eq!(name.len(), 14);
    assert!(branch::validate_branch_name(&name).is_ok());
}

#[test]
fn hex_lower_formats_bytes() {
    assert_eq!(hex_lower(&[0xab, 0x01, 0x9f, 0x00]), "ab019f00");
}

#[tokio::test]
async fn unique_branch_retries_on_collision() {
    let tmp = init_repo_with_commit();
    branch::create_branch(tmp.path(), "clone-00000000").expect("seed branch");
    let result = create_unique_branch_from_candidates(
        tmp.path().to_path_buf(),
        ["clone-00000000".to_string(), "clone-11111111".to_string()].to_vec(),
    )
    .await
    .expect("create branch");
    assert_eq!(result, "clone-11111111");
}

#[tokio::test]
async fn unique_branch_fails_after_three_collisions() {
    let tmp = init_repo_with_commit();
    for name in ["clone-00000000", "clone-11111111", "clone-22222222"] {
        branch::create_branch(tmp.path(), name).expect("seed branch");
    }
    let result = create_unique_branch_from_candidates(
        tmp.path().to_path_buf(),
        ["clone-00000000", "clone-11111111", "clone-22222222"]
            .map(str::to_string)
            .to_vec(),
    )
    .await;
    assert_eq!(result, Err(branch::CreateBranchError::AlreadyExists));
}

#[tokio::test]
async fn cleanup_refuses_branch_linked_to_another_session() {
    let root_id = Uuid::new_v4().to_string();
    let clone_a_id = Uuid::new_v4().to_string();
    let clone_b_id = Uuid::new_v4().to_string();
    let branch_name = "clone-11111111";
    let repo = init_repo_with_commit();
    super::session_store::save(&session(&root_id, None, None))
        .await
        .expect("save root");
    super::session_store::save(&session(&clone_a_id, Some(&root_id), Some(branch_name)))
        .await
        .expect("save clone a");
    super::session_store::save(&session(&clone_b_id, Some(&root_id), Some(branch_name)))
        .await
        .expect("save clone b");
    super::session_tabs::add_clone_tab(&root_id, &clone_a_id, "m1", CloneMode::Cut)
        .await
        .expect("add clone a tab");
    super::session_tabs::add_clone_tab(&root_id, &clone_b_id, "m1", CloneMode::Cut)
        .await
        .expect("add clone b tab");
    let tabs = super::session_tabs::list(&root_id).await.expect("tabs");
    let tab_id = tabs
        .tabs
        .iter()
        .find(|tab| tab.session_id == clone_a_id)
        .expect("clone a tab")
        .tab_id
        .clone();

    let result =
        close_tab_with_branch_cleanup(&root_id, &tab_id, repo.path(), Some("master")).await;

    assert_eq!(
        result.unwrap_err(),
        BRANCH_LINKED_TO_OTHER_SESSION_ERROR.to_string()
    );
    let saved = super::session_store::get(&clone_a_id)
        .await
        .expect("clone remains");
    assert_eq!(saved.git_branch.as_deref(), Some(branch_name));
    let tabs = super::session_tabs::list(&root_id)
        .await
        .expect("tabs remain");
    assert!(tabs.tabs.iter().any(|tab| tab.session_id == clone_a_id));

    let _ = super::session_tabs::remove_session_from_tabs(&clone_a_id).await;
    let _ = super::session_tabs::remove_session_from_tabs(&clone_b_id).await;
    let _ = super::session_tabs::remove_session_from_tabs(&root_id).await;
    let _ = super::session_store::delete_one(&clone_a_id).await;
    let _ = super::session_store::delete_one(&clone_b_id).await;
    let _ = super::session_store::delete_one(&root_id).await;
}
