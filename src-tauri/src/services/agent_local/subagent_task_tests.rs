#[cfg(test)]
mod tests {
    use crate::services::agent_local::subagent_working_dir::create_coder_worktree_for_test;
    use uuid::Uuid;

    fn temp_dir() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("cl-go-subagent-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).expect("create test dir");
        path
    }

    #[tokio::test]
    async fn test_coder_worktree_creation_failure_is_blocking() {
        let project = temp_dir();
        let result = create_coder_worktree_for_test(&project, "child-session").await;

        assert!(
            result.is_err(),
            "un sous-agent coder ne doit pas retomber dans le dossier principal si le worktree échoue"
        );

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn queued_followup_keeps_session_running() {
        let status = super::super::subagent_task::effective_session_status("completed", true);

        assert_eq!(status, "running");
    }

    #[test]
    fn final_run_keeps_completed_status() {
        let status = super::super::subagent_task::effective_session_status("completed", false);

        assert_eq!(status, "completed");
    }

    #[test]
    fn running_activity_label_is_not_completed() {
        assert_eq!(
            super::super::subagent_task::final_activity_label("running"),
            "En cours"
        );
    }
}
