#[cfg(test)]
mod tests {
    use crate::services::agent_local::subagent_registry::{
        active_children_for_parent, cancel_one, get_or_create_run_id, get_run_id_for_child,
        register, unregister,
    };
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

        // --- get_run_id_for_child ---
        let parent = uid();
        let child = uid();
        let run_id = register(&parent, &child, CancellationToken::new())
            .await
            .unwrap();
        let fetched = get_run_id_for_child(&child).await;
        assert_eq!(fetched, Some(run_id));
        assert_eq!(
            active_children_for_parent(&parent).await,
            vec![child.clone()]
        );
        unregister(&child).await;
        assert_eq!(get_run_id_for_child(&child).await, None);
        assert!(active_children_for_parent(&parent).await.is_empty());

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
}
