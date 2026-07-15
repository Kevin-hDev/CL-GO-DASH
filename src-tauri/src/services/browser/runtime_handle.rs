use super::lifecycle::{Lifecycle, RuntimePhase};
use serde::Serialize;
use std::sync::{Arc, Mutex};

pub(super) const CEF_VERSION: &str = "150.0.0+150.0.10";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum BrowserCapability {
    Ready { engine_version: String },
    Unavailable,
    Hidden,
}

#[derive(Clone, Default)]
pub struct BrowserRuntimeHandle {
    lifecycle: Arc<Mutex<Lifecycle>>,
}

impl BrowserRuntimeHandle {
    #[cfg(test)]
    pub(super) fn is_ready(&self) -> bool {
        self.lifecycle
            .lock()
            .map(|lifecycle| lifecycle.phase() == RuntimePhase::Running)
            .unwrap_or(false)
    }

    pub fn capability(&self) -> BrowserCapability {
        let Ok(lifecycle) = self.lifecycle.lock() else {
            return BrowserCapability::Unavailable;
        };
        match lifecycle.phase() {
            RuntimePhase::Running => BrowserCapability::Ready {
                engine_version: CEF_VERSION.to_string(),
            },
            _ => BrowserCapability::Unavailable,
        }
    }

    pub(super) fn mark_application_prepared(&self) -> bool {
        self.lifecycle
            .lock()
            .map(|mut lifecycle| lifecycle.mark_application_prepared())
            .unwrap_or(false)
    }

    pub(super) fn mark_running(&self) -> bool {
        self.lifecycle
            .lock()
            .map(|mut lifecycle| lifecycle.mark_running())
            .unwrap_or(false)
    }

    pub(super) fn mark_failed(&self) -> bool {
        self.lifecycle
            .lock()
            .map(|mut lifecycle| lifecycle.mark_failed())
            .unwrap_or(false)
    }

    pub(super) fn begin_stopping(&self) -> bool {
        self.lifecycle
            .lock()
            .map(|mut lifecycle| lifecycle.begin_stopping())
            .unwrap_or(false)
    }

    pub(super) fn mark_stopped(&self) -> bool {
        self.lifecycle
            .lock()
            .map(|mut lifecycle| lifecycle.mark_stopped())
            .unwrap_or(false)
    }
}
