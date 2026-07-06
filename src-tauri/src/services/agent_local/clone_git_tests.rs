use super::*;
use git2::{Repository, Signature};

fn init_repo_with_commit() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    std::fs::write(tmp.path().join("README.md"), "init").expect("write file");
    let mut index = repo.index().expect("index");
    index.add_path(std::path::Path::new("README.md")).expect("add");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    let sig = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("commit");
    tmp
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
        ["clone-00000000", "clone-11111111", "clone-22222222"].map(str::to_string).to_vec(),
    )
    .await;
    assert_eq!(result, Err(branch::CreateBranchError::AlreadyExists));
}

#[test]
fn ensure_clone_belongs_to_root_accepts_clone_of_clone() {
    // Un clone-de-clone a `clone_root_session_id == root` mais
    // `clone_parent_session_id != root` (pointe vers le clone intermédiaire).
    // Le check doit accepter ce cas grâce à la comparaison sur la racine.
    use crate::services::agent_local::types_session::AgentSession;
    use chrono::Utc;

    let root_id = "root-aaaaaaaa-1111-1111-1111-111111111111";
    let intermediate_id = "clone-bbbbbbbb-2222-2222-2222-222222222222";
    let intermediate = AgentSession {
        id: intermediate_id.into(),
        name: "Clone intermediate".into(),
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
        clone_parent_session_id: Some(root_id.into()),
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: vec![],
        clone_modified_files: vec![],
        clone_root_session_id: Some(root_id.into()),
        git_branch: None,
    };

    // Le clone intermédiaire lui-même appartient bien au groupe.
    assert!(ensure_clone_belongs_to_root(&intermediate, root_id).is_ok());
    assert!(ensure_clone_belongs_to_root_string(&intermediate, root_id).is_ok());

    // Simule un clone-de-clone : parent direct = clone intermédiaire,
    // racine = root. Le check doit quand même accepter.
    let mut clone_of_clone = intermediate.clone();
    clone_of_clone.id = "clone-of-clone".into();
    clone_of_clone.clone_parent_session_id = Some(intermediate_id.into());
    clone_of_clone.clone_root_session_id = Some(root_id.into());

    assert!(ensure_clone_belongs_to_root(&clone_of_clone, root_id).is_ok());
    assert!(ensure_clone_belongs_to_root_string(&clone_of_clone, root_id).is_ok());

    // Un clone qui n'appartient pas au groupe doit être refusé.
    let other_root = "other-cccccccc-3333-3333-3333-333333333333";
    assert!(ensure_clone_belongs_to_root(&clone_of_clone, other_root).is_err());
}
