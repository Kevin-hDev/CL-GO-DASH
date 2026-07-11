#[cfg(test)]
mod tests {
    use crate::services::agent_local::subagent_registry::{
        active_children_for_parent, cancel_one, capacity_error, complete_child, consume_terminal,
        get_or_create_run_id, get_run_id_for_child, parent_snapshot, register,
        release_run_claim, subscribe_for_parent, terminal_state_for_parent, unregister,
        SubagentTerminalKind, PRODUCTION_MAX_TERMINAL_PARENTS,
    };
    use crate::services::agent_local::subagent_registry_test_support::meta;
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
        assert_eq!(PRODUCTION_MAX_TERMINAL_PARENTS, 16);

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

        // --- terminal state survives last child and waves do not reuse stale state ---
        let parent = uid();
        let first_child = uid();
        register(&parent, &first_child, CancellationToken::new())
            .await
            .unwrap();
        complete_child(
            &first_child,
            SubagentTerminalKind::ReportPersistenceFailed,
        )
        .await
        .unwrap();

        assert!(active_children_for_parent(&parent).await.is_empty());
        let first_receiver = subscribe_for_parent(&parent)
            .await
            .expect("last-child terminal state remains subscribable");
        let first_state = *first_receiver.borrow();
        assert_eq!(first_state.sequence, 1);
        assert!(first_state.report_persistence_failed);
        assert_eq!(terminal_state_for_parent(&parent).await, Some(first_state));
        assert!(register(&parent, &uid(), CancellationToken::new())
            .await
            .is_err());
        assert!(consume_terminal(&parent, first_state.generation).await);
        assert!(subscribe_for_parent(&parent).await.is_none());

        let second_child = uid();
        let second_run = register(&parent, &second_child, CancellationToken::new())
            .await
            .unwrap();
        let mut second_receiver = subscribe_for_parent(&parent)
            .await
            .expect("second-wave receiver");
        let second_generation = second_receiver.borrow().generation;
        assert_ne!(second_generation, first_state.generation);
        assert_eq!(second_receiver.borrow().sequence, 0);
        complete_child(&second_child, SubagentTerminalKind::ReportPersisted)
            .await
            .unwrap();
        second_receiver.changed().await.expect("second-wave signal");
        assert_eq!(second_receiver.borrow().sequence, 1);
        assert!(!second_receiver.borrow().report_persistence_failed);
        let restarted_child = uid();
        let restarted_run = register(&parent, &restarted_child, CancellationToken::new())
            .await
            .expect("successful pending terminal permits explicit restart");
        assert_ne!(second_run, restarted_run);
        assert_eq!(second_receiver.borrow().sequence, 1);
        assert!(consume_terminal(&parent, second_generation).await);
        assert_eq!(second_receiver.borrow().sequence, 0);
        unregister(&restarted_child).await;

        // --- removal and terminal notification are one registry transition ---
        let parent = uid();
        let child = uid();
        register(&parent, &child, CancellationToken::new())
            .await
            .unwrap();
        let completing_child = child.clone();
        let completion = tokio::spawn(async move {
            complete_child(
                &completing_child,
                SubagentTerminalKind::ReportPersisted,
            )
            .await
            .unwrap();
        });
        let terminal = loop {
            let snapshot = parent_snapshot(&parent).await;
            assert!(
                !snapshot.active_child_ids.is_empty()
                    || snapshot
                        .terminal_state
                        .is_some_and(|state| state.sequence > 0)
            );
            if let Some(state) = snapshot.terminal_state.filter(|state| state.sequence > 0) {
                break state;
            }
            tokio::task::yield_now().await;
        };
        completion.await.unwrap();
        assert!(consume_terminal(&parent, terminal.generation).await);

        // --- cancel_one ---
        let parent = uid();
        let child = uid();
        let token = CancellationToken::new();
        register(&parent, &child, token.clone()).await.unwrap();
        assert!(cancel_one(&child).await);
        assert!(token.is_cancelled());
        unregister(&child).await;

        // --- capacity rules without saturating the shared test registry ---
        assert!(capacity_error(MAX_TOTAL, 0).is_some());
        assert!(capacity_error(0, MAX_PER_PARENT).is_some());
        assert!(capacity_error(MAX_TOTAL - 1, MAX_PER_PARENT - 1).is_none());

    }

}
