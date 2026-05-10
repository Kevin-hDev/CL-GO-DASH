use std::collections::HashMap;
use std::sync::Arc;

use tauri::Emitter;
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;

use crate::models::{ChannelAccountConfig, GatewayConfig};
use super::agent_bridge::GatewayAgentBridge;
use super::channels::telegram::TelegramAdapter;
use super::channels::{ChannelAdapter, ChannelContext, InboundMessage};
use super::types::{ChannelHealthEntry, ChannelKey, ChannelStatus, GatewayHealth};

struct ChannelEntry {
    status: ChannelStatus,
    cancel: CancellationToken,
    error: Option<String>,
}

pub struct GatewayState {
    pub(crate) channels: HashMap<ChannelKey, ChannelEntry>,
    adapters: HashMap<ChannelKey, Arc<dyn ChannelAdapter>>,
    config: GatewayConfig,
    pub(crate) cancel: CancellationToken,
}

fn build_health(state: &GatewayState) -> GatewayHealth {
    let channels = state.channels.iter()
        .map(|(key, entry)| ChannelHealthEntry {
            channel_id: key.channel_id.clone(),
            account_id: key.account_id.clone(),
            status: entry.status,
            error: entry.error.clone(),
        })
        .collect();
    GatewayHealth { running: !state.cancel.is_cancelled(), channels }
}

pub struct GatewayService {
    pub(crate) state: Arc<RwLock<GatewayState>>,
}

impl GatewayService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(GatewayState {
                channels: HashMap::new(),
                adapters: HashMap::new(),
                config: GatewayConfig::default(),
                cancel: CancellationToken::new(),
            })),
        }
    }

    pub async fn start(&self, config: GatewayConfig, app: tauri::AppHandle) {
        let mut state = self.state.write().await;
        state.cancel = CancellationToken::new();
        state.channels.clear();
        state.adapters.clear();
        state.config = config.clone();

        let (tx, mut rx) = mpsc::channel::<InboundMessage>(256);

        self.start_channel_accounts(&mut state, "telegram", &config.channels.telegram, &tx, &app);
        self.start_channel_accounts(&mut state, "slack", &config.channels.slack, &tx, &app);
        self.start_channel_accounts(&mut state, "discord", &config.channels.discord, &tx, &app);

        let bridge = Arc::new(GatewayAgentBridge::new());
        let state_ref = self.state.clone();
        let app_ref = app.clone();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let _ = app_ref.emit("gateway-message-received", &msg);
                let adapter = {
                    let s = state_ref.read().await;
                    s.adapters.get(&msg.channel_key).cloned()
                };
                if let Some(adapter) = adapter {
                    let b = Arc::clone(&bridge);
                    let a = app_ref.clone();
                    tokio::spawn(async move {
                        if let Err(e) = b.process(msg, adapter, a).await {
                            eprintln!("[gateway-bridge] {e}");
                        }
                    });
                }
            }
        });

        let _ = app.emit("gateway-status-changed", build_health(&state));
    }

    fn start_channel_accounts(
        &self, state: &mut GatewayState, channel_id: &str,
        accounts: &[ChannelAccountConfig], tx: &mpsc::Sender<InboundMessage>,
        app: &tauri::AppHandle,
    ) {
        for acc in accounts {
            if !acc.enabled { continue; }
            let key = ChannelKey::new(channel_id, &acc.account_id);
            let child_cancel = state.cancel.child_token();
            let adapter: Arc<dyn ChannelAdapter> = match channel_id {
                "telegram" => Arc::new(TelegramAdapter::new()),
                "slack" => Arc::new(super::channels::slack::SlackAdapter::new()),
                "discord" => Arc::new(super::channels::discord::DiscordAdapter::new()),
                _ => continue,
            };
            state.adapters.insert(key.clone(), Arc::clone(&adapter));
            let ctx = ChannelContext {
                key: key.clone(), config: acc.clone(),
                cancel: child_cancel.clone(), app: app.clone(),
            };
            state.channels.insert(key.clone(), ChannelEntry {
                status: ChannelStatus::Starting, cancel: child_cancel, error: None,
            });
            let sender = tx.clone();
            let state_arc = self.state.clone();
            let key_clone = key;
            let app_clone = app.clone();
            tokio::spawn(async move {
                match adapter.start(ctx, sender).await {
                    Ok(_handle) => {
                        let mut s = state_arc.write().await;
                        if let Some(e) = s.channels.get_mut(&key_clone) { e.status = ChannelStatus::Running; }
                        let _ = app_clone.emit("gateway-status-changed", build_health(&s));
                    }
                    Err(e) => {
                        let mut s = state_arc.write().await;
                        if let Some(entry) = s.channels.get_mut(&key_clone) {
                            entry.status = ChannelStatus::Error;
                            entry.error = Some(e.message);
                        }
                        let _ = app_clone.emit("gateway-status-changed", build_health(&s));
                    }
                }
            });
        }
    }

    pub async fn stop(&self) {
        let mut state = self.state.write().await;
        state.cancel.cancel();
        for entry in state.channels.values_mut() {
            entry.cancel.cancel();
            entry.status = ChannelStatus::Stopping;
        }
    }

    pub async fn health(&self) -> GatewayHealth { let s = self.state.read().await; build_health(&s) }
    pub async fn is_enabled(&self) -> bool {
        let s = self.state.read().await;
        s.config.enabled && !s.cancel.is_cancelled()
    }
    pub async fn config(&self) -> GatewayConfig { self.state.read().await.config.clone() }
    pub async fn update_config(&self, config: GatewayConfig) { self.state.write().await.config = config; }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
