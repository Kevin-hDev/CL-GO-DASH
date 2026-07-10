#[cfg(test)]
mod tests {
    use crate::services::agent_local::subagent_registry::{
        active_children_for_parent, cancel_one, get_or_create_run_id, get_run_id_for_child,
        register, release_run_claim, subscribe_for_parent, terminal_notifier_for_child, unregister,
        SubagentTerminalKind,
    };
    use crate::services::agent_local::types_session::AgentSessionMeta;
    use chrono::Utc;
    use tokio_util::sync::CancellationToken;

    const MAX_PER_PARENT: usize = 4;
    const MAX_TOTAL: usize = 8;

    fn uid() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    // All tests run in a single async test to avoid state conflicts
    // on the global static registry shared across parallel tokio tests.
    #[tokio::test]
    async fn test_registry_all() {
        // --- get_or_create_run_id ---
        let p1 = uid();
        let p2 = uid();
        let r1a = get_or_create_run_id(&p1).await;
        let r1b = get_or_create_run_id(&p1).await;
        let r2 = get_or_create_run_id(&p2).await;
        assert_eq!(r1a, r1b);
        assert_ne!(r1a, r2);
        release_run_claim(&p1, &r1a).await;
        release_run_claim(&p1, &r1a).await;
        release_run_claim(&p2, &r2).await;

        // --- get_run_id_for_child ---
        let parent = uid();
        let child = uid();
        let run_id = register(&parent, &child, CancellationToken::new())
            .await
            .unwrap();
        let fetched = get_run_id_for_child(&child).await;
        assert_eq!(fetched, Some(run_id.clone()));
        let normalized = crate::services::agent_local::subagent_live_state::normalize_meta(meta(
            &child,
            "completed",
        ))
        .await;
        assert_eq!(normalized.subagent_status.as_deref(), Some("running"));
        assert_eq!(normalized.subagent_run_id.as_deref(), Some(run_id.as_str()));
        assert_eq!(
            active_children_for_parent(&parent).await,
            vec![child.clone()]
        );
        unregister(&child).await;
        assert_eq!(get_run_id_for_child(&child).await, None);
        assert!(active_children_for_parent(&parent).await.is_empty());

        // --- shared completion signal, including notify-before-subscribe/wait ---
        let parent = uid();
        let first_child = uid();
        let second_child = uid();
        register(&parent, &first_child, CancellationToken::new())
            .await
            .unwrap();
        register(&parent, &second_child, CancellationToken::new())
            .await
            .unwrap();
        let first_notifier = terminal_notifier_for_child(&first_child)
            .await
            .expect("first notifier");
        unregister(&first_child).await;
        first_notifier.notify(SubagentTerminalKind::ReportPersisted);

        let mut receiver = subscribe_for_parent(&parent)
            .await
            .expect("shared receiver");
        assert_eq!(receiver.borrow().sequence, 1);
        assert!(!receiver.borrow().report_persistence_failed);

        let second_notifier = terminal_notifier_for_child(&second_child)
            .await
            .expect("second notifier");
        unregister(&second_child).await;
        second_notifier.notify(SubagentTerminalKind::ReportPersistenceFailed);
        tokio::time::timeout(std::time::Duration::from_secs(1), receiver.changed())
            .await
            .expect("signal before wait is retained")
            .expect("sender remains available through notifier");
        assert_eq!(receiver.borrow().sequence, 2);
        assert!(receiver.borrow().report_persistence_failed);

        // --- cancel_one ---
        let parent = uid();
        let child = uid();
        let token = CancellationToken::new();
        register(&parent, &child, token.clone()).await.unwrap();
        assert!(cancel_one(&child).await);
        assert!(token.is_cancelled());
        unregister(&child).await;

        // --- max_per_parent limit ---
        let parent = uid();
        let mut children = vec![];
        for _ in 0..MAX_PER_PARENT {
            let c = uid();
            register(&parent, &c, CancellationToken::new())
                .await
                .unwrap();
            children.push(c);
        }
        let extra = uid();
        assert!(register(&parent, &extra, CancellationToken::new())
            .await
            .is_err());
        for c in &children {
            unregister(c).await;
        }

        // --- max_total limit ---
        let mut all_children = vec![];
        for _ in 0..MAX_TOTAL {
            let p = uid();
            let c = uid();
            register(&p, &c, CancellationToken::new()).await.unwrap();
            all_children.push(c);
        }
        let extra_p = uid();
        let extra_c = uid();
        assert!(register(&extra_p, &extra_c, CancellationToken::new())
            .await
            .is_err());
        for c in &all_children {
            unregister(c).await;
        }
    }

    fn meta(id: &str, status: &str) -> AgentSessionMeta {
        AgentSessionMeta {
            id: id.into(),
            name: "Geminitor".into(),
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
            archived_at: None,
            model: "llama3".into(),
            provider: "ollama".into(),
            thinking_enabled: false,
            reasoning_mode: None,
            message_count: 0,
            is_heartbeat: false,
            is_gateway: false,
            gateway_channel_key: None,
            project_id: None,
            parent_session_id: Some("parent".into()),
            subagent_type: Some("explorer".into()),
            subagent_status: Some(status.into()),
            subagent_run_id: Some("saved-run".into()),
            subagent_description: Some("Analyse".into()),
            subagent_color_key: Some("geminitor".into()),
            subagent_summary: None,
            subagent_last_activity: None,
            clone_parent_session_id: None,
            clone_parent_message_id: None,
            clone_mode: None,
            clone_root_session_id: None,
            git_branch: None,
        }
    }
}
