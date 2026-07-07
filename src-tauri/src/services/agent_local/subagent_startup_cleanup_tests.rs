use super::cleanup_orphans;
use crate::services::agent_local::session_store;
use crate::services::agent_local::session_subagents::mark_status;
use crate::services::agent_local::subagent_worktree::create_for_child;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_git_repo() -> PathBuf {
    let path = std::env::temp_dir()
        .join(format!("cl-go-subagent-cleanup-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&path).expect("create test dir");
    std::process::Command::new("git")
        .args(["init"])
        .arg(&path)
        .output()
        .expect("git init");
    // Commit initial requis pour pouvoir attacher un worktree sur HEAD.
    std::fs::write(path.join("README.md"), "init").expect("write readme");
    std::process::Command::new("git")
        .args(["-C"])
        .arg(&path)
        .args(["add", "."])
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["-C"])
        .arg(&path)
        .args([
            "-c",
            "user.email=test@cl-go.test",
            "-c",
            "user.name=Test",
            "commit",
            "-m",
            "init",
        ])
        .output()
        .expect("git commit");
    path
}

#[tokio::test]
async fn cleanup_reclassifies_running_orphans_and_removes_worktrees() {
    // Dépôt git réel pour pouvoir créer un vrai worktree.
    let repo = temp_git_repo();

    // Session A : sous-agent "running" orphelin avec un vrai worktree.
    let mut orphan = session_store::create_full("orphan", "llama3", "ollama", false, None)
        .await
        .expect("create orphan session");
    orphan.parent_session_id = Some(Uuid::new_v4().to_string());
    orphan.subagent_status = Some("running".to_string());

    let worktree = create_for_child(&repo, &orphan.id).await.expect("worktree");
    assert!(worktree.is_dir(), "le worktree doit exister avant cleanup");
    orphan.subagent_worktree = Some(worktree.to_string_lossy().to_string());
    session_store::save(&orphan).await.expect("save orphan");

    // Session B : sous-agent déjà "completed", ne doit pas être touché.
    let mut done = session_store::create_full("done", "llama3", "ollama", false, None)
        .await
        .expect("create done session");
    done.parent_session_id = Some(Uuid::new_v4().to_string());
    done.subagent_status = Some("completed".to_string());
    session_store::save(&done).await.expect("save done");

    cleanup_orphans().await;

    let after_orphan = session_store::get(&orphan.id).await.expect("reload orphan");
    assert_eq!(
        after_orphan.subagent_status.as_deref(),
        Some("interrupted"),
        "un sous-agent running orphan doit devenir interrupted"
    );
    assert!(
        !worktree.exists(),
        "le worktree de l'orphelin doit être supprimé"
    );

    let after_done = session_store::get(&done.id).await.expect("reload done");
    assert_eq!(
        after_done.subagent_status.as_deref(),
        Some("completed"),
        "un sous-agent completed ne doit pas être modifié"
    );

    // Nettoyage.
    let _ = session_store::delete(&orphan.id).await;
    let _ = session_store::delete(&done.id).await;
    let _ = std::fs::remove_dir_all(&repo);
}

#[tokio::test]
async fn mark_status_persists_interrupted() {
    let mut session = session_store::create_full("s", "llama3", "ollama", false, None)
        .await
        .expect("create session");
    session.parent_session_id = Some(Uuid::new_v4().to_string());
    session.subagent_status = Some("running".to_string());
    session_store::save(&session).await.expect("save");

    mark_status(&session.id, "interrupted")
        .await
        .expect("mark_status interrupted");

    let reloaded = session_store::get(&session.id).await.expect("reload");
    assert_eq!(reloaded.subagent_status.as_deref(), Some("interrupted"));

    let _ = session_store::delete(&session.id).await;
}
