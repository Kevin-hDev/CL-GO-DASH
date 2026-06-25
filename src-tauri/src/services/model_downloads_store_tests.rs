#[cfg(test)]
mod tests {
    use crate::services::model_downloads_store::{ModelDownloadManager, ProgressUpdate};
    use crate::services::model_downloads_types::{
        ModelDownloadKind, ModelDownloadPhase, ModelDownloadStatus, MAX_ACTIVE_DOWNLOADS,
    };

    #[tokio::test]
    async fn rejects_second_active_download() {
        let manager = ModelDownloadManager::new();
        let first = manager
            .start(ModelDownloadKind::Ollama, "llama3:latest".into(), false)
            .await;
        assert!(first.is_ok());

        let second = manager
            .start(ModelDownloadKind::Forecast, "chronos-tiny".into(), false)
            .await;
        assert_eq!(second.unwrap_err(), "model-download-already-active");
        assert_eq!(manager.list().await.len(), MAX_ACTIVE_DOWNLOADS);
    }

    #[tokio::test]
    async fn cancel_marks_token_then_finish_sets_cancelled_status() {
        let manager = ModelDownloadManager::new();
        let (state, cancel) = manager
            .start(ModelDownloadKind::Forecast, "chronos-tiny".into(), false)
            .await
            .unwrap();

        manager.cancel(&state.id).await.unwrap();
        assert!(cancel.is_cancelled());

        let states = manager
            .finish(&state.id, ModelDownloadStatus::Cancelled, None)
            .await;
        assert_eq!(states[0].status, ModelDownloadStatus::Cancelled);
    }

    #[tokio::test]
    async fn completed_download_is_removed_before_next_start() {
        let manager = ModelDownloadManager::new();
        let (state, _) = manager
            .start(ModelDownloadKind::Ollama, "llama3:latest".into(), false)
            .await
            .unwrap();
        manager
            .progress(
                &state.id,
                ProgressUpdate {
                    phase: ModelDownloadPhase::Downloading,
                    downloaded: 1,
                    total: 2,
                    percent: 50,
                },
            )
            .await;
        manager
            .finish(&state.id, ModelDownloadStatus::Completed, None)
            .await;

        let (next, _) = manager
            .start(ModelDownloadKind::Forecast, "chronos-tiny".into(), false)
            .await
            .unwrap();
        let states = manager.list().await;
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].id, next.id);
    }
}
