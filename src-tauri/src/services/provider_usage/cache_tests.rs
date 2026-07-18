use super::*;

#[tokio::test]
async fn cache_is_bounded_to_connection_limit() {
    clear().await;
    for index in 0..=super::super::types::CONNECTION_LIMIT {
        put(
            &format!("connection-{index}"),
            RemoteData {
                fetched_at: index as i64,
                ..Default::default()
            },
        )
        .await;
    }
    assert_eq!(len().await, super::super::types::CONNECTION_LIMIT);
    assert!(get("connection-0").await.is_none());
}
