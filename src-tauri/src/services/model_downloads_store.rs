use super::model_downloads_types::{
    ModelDownloadKind, ModelDownloadPhase, ModelDownloadState, ModelDownloadStatus,
    MAX_ACTIVE_DOWNLOADS,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ModelDownloadManager {
    inner: Arc<Mutex<HashMap<String, ActiveDownload>>>,
}

#[derive(Debug, Clone)]
struct ActiveDownload {
    state: ModelDownloadState,
    cancel: CancellationToken,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub phase: ModelDownloadPhase,
    pub downloaded: u64,
    pub total: u64,
    pub percent: u8,
}

impl ModelDownloadManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn inner_clone(&self) -> Self {
        self.clone()
    }

    pub async fn start(
        &self,
        kind: ModelDownloadKind,
        model_id: String,
        is_update: bool,
    ) -> Result<(ModelDownloadState, CancellationToken), String> {
        let mut downloads = self.inner.lock().await;
        if downloads
            .values()
            .filter(|item| item.state.status == ModelDownloadStatus::Running)
            .count()
            >= MAX_ACTIVE_DOWNLOADS
        {
            return Err("model-download-already-active".into());
        }
        downloads.retain(|_, item| item.state.status == ModelDownloadStatus::Running);

        let id = Uuid::new_v4().to_string();
        let cancel = CancellationToken::new();
        let state = ModelDownloadState::new(kind, model_id, is_update, id.clone());
        downloads.insert(
            id,
            ActiveDownload {
                state: state.clone(),
                cancel: cancel.clone(),
            },
        );
        Ok((state, cancel))
    }

    pub async fn list(&self) -> Vec<ModelDownloadState> {
        self.inner
            .lock()
            .await
            .values()
            .map(|item| item.state.clone())
            .collect()
    }

    pub async fn progress(&self, id: &str, update: ProgressUpdate) -> Vec<ModelDownloadState> {
        let mut downloads = self.inner.lock().await;
        if let Some(active) = downloads.get_mut(id) {
            active.state.phase = update.phase;
            active.state.downloaded = update.downloaded;
            active.state.total = update.total;
            active.state.percent = update.percent.min(100);
        }
        downloads.values().map(|item| item.state.clone()).collect()
    }

    pub async fn finish(
        &self,
        id: &str,
        status: ModelDownloadStatus,
        error_key: Option<&str>,
    ) -> Vec<ModelDownloadState> {
        let mut downloads = self.inner.lock().await;
        if let Some(active) = downloads.get_mut(id) {
            active.state.status = status;
            active.state.error_key = error_key.map(str::to_string);
            if status == ModelDownloadStatus::Completed {
                active.state.phase = ModelDownloadPhase::Completed;
                active.state.percent = 100;
            }
        }
        downloads.values().map(|item| item.state.clone()).collect()
    }

    pub async fn cancel(&self, id: &str) -> Result<(), String> {
        let downloads = self.inner.lock().await;
        let active = downloads
            .get(id)
            .ok_or_else(|| "model-download-not-found".to_string())?;
        active.cancel.cancel();
        Ok(())
    }
}

impl Default for ModelDownloadManager {
    fn default() -> Self {
        Self::new()
    }
}
