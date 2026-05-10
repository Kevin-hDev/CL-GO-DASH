use std::sync::Arc;

use tauri::Emitter;
use tokio_util::sync::CancellationToken;

use crate::commands::agent_chat_task::run_stream_task;
use crate::models::{ChannelAccountConfig, GatewayConfig};
use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::gateway::channels::{ChannelAdapter, InboundMessage, OutboundMessage};
use crate::services::gateway::message_convert;
use crate::services::gateway::security::{allowlist::Allowlist, validation};
use crate::services::gateway::session_map;
use crate::services::gateway::stream_capture;

#[derive(Debug)]
pub enum BridgeError {
    Blocked(String),
    SessionError(String),
    AgentError(String),
    SendError(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocked(r) => write!(f, "blocked: {r}"),
            Self::SessionError(e) => write!(f, "session: {e}"),
            Self::AgentError(e) => write!(f, "agent: {e}"),
            Self::SendError(e) => write!(f, "send: {e}"),
        }
    }
}

pub struct GatewayAgentBridge;

impl GatewayAgentBridge {
    pub fn new() -> Self { Self }

    fn read_config() -> GatewayConfig {
        crate::services::config::read_config().map(|c| c.gateway).unwrap_or_default()
    }

    pub async fn process(
        &self, msg: InboundMessage, adapter: Arc<dyn ChannelAdapter>, app: tauri::AppHandle,
    ) -> Result<(), BridgeError> {
        let vr = validation::validate_message(&msg.content);
        if !vr.valid {
            return Err(BridgeError::Blocked(vr.reason.unwrap_or_default()));
        }

        let config = Self::read_config();
        let account_cfg = find_account_config(&config, &msg);

        if let Some(cfg) = &account_cfg {
            let al = Allowlist::from_list(&cfg.allowlist, false);
            if !al.contains(&msg.user_id) {
                return Err(BridgeError::Blocked("user not in allowlist".into()));
            }
        }

        let channel_key = build_external_key(&msg);
        let (provider, model) = resolve_provider_model(&account_cfg, &config);
        let session_id = find_or_create_session(&msg, &channel_key, &provider, &model, &app).await?;

        sync_session_model(&session_id, &provider, &model).await;

        let session = session_store::get(&session_id).await.map_err(BridgeError::SessionError)?;
        let mut messages = message_convert::build_chat_messages(&session);
        let history_len = messages.len();
        messages.push(message_convert::new_user_message(&msg.content));

        let emitter = AgentEventEmitter::new(app.clone(), session_id.clone());
        let final_messages = run_stream_task(
            emitter, session_id.clone(), model, messages,
            vec![], false, provider, None, None, None,
            Some("auto".to_string()), CancellationToken::new(),
        ).await.map_err(BridgeError::AgentError)?;

        let new_assistant_messages: Vec<_> = {
            let mut non_system = final_messages.iter().filter(|m| m.role != "system");
            for _ in 0..history_len { non_system.next(); }
            non_system.next();
            non_system.filter_map(message_convert::chat_to_agent_message).collect()
        };

        let mut to_persist = vec![message_convert::new_user_agent_message(&msg.content)];
        to_persist.extend(new_assistant_messages);
        let _ = session_store::add_messages(&session_id, to_persist, 0).await;
        let _ = app.emit("wakeup-completed", serde_json::json!({ "session_id": &session_id }));

        if let Some(reply) = stream_capture::extract_final_reply(&final_messages) {
            let max_utf16 = adapter.capabilities().max_message_chars;
            for chunk in stream_capture::prepare_for_channel(&reply, max_utf16) {
                adapter.send(OutboundMessage {
                    chat_id: msg.chat_id.clone(), content: chunk,
                    reply_to: Some(msg.message_id.clone()),
                }).await.map_err(|e| BridgeError::SendError(e.message))?;
            }
        }
        Ok(())
    }
}

fn resolve_provider_model(account: &Option<ChannelAccountConfig>, config: &GatewayConfig) -> (String, String) {
    if let Some(acc) = account {
        if !acc.provider.is_empty() && !acc.model.is_empty() {
            return (acc.provider.clone(), acc.model.clone());
        }
    }
    (config.default_provider.clone(), config.default_model.clone())
}

async fn sync_session_model(session_id: &str, provider: &str, model: &str) {
    if let Ok(session) = session_store::get(session_id).await {
        if session.provider != provider || session.model != model {
            let _ = session_store::update_model(session_id, model, provider).await;
        }
    }
}

fn find_account_config(config: &GatewayConfig, msg: &InboundMessage) -> Option<ChannelAccountConfig> {
    let accounts = match msg.channel_key.channel_id.as_str() {
        "telegram" => &config.channels.telegram,
        "slack" => &config.channels.slack,
        "discord" => &config.channels.discord,
        _ => return None,
    };
    accounts.iter().find(|a| a.account_id == msg.channel_key.account_id).cloned()
}

fn build_external_key(msg: &InboundMessage) -> String {
    format!("{}/{}/{}", msg.channel_key.channel_id, msg.channel_key.account_id, msg.user_id)
}

async fn find_or_create_session(
    msg: &InboundMessage, channel_key: &str, provider: &str, model: &str,
    app: &tauri::AppHandle,
) -> Result<String, BridgeError> {
    if let Some(id) = session_map::find(channel_key).await {
        if session_store::get(&id).await.is_ok() {
            return Ok(id);
        }
    }
    let label = match msg.channel_key.channel_id.as_str() {
        "telegram" => "TG", "slack" => "SL", "discord" => "DC", _ => "GW",
    };
    let name = format!("[{label}] {}", msg.user_id);
    let session = session_store::create_gateway(&name, model, provider, channel_key.to_string())
        .await.map_err(BridgeError::SessionError)?;
    session_map::insert(channel_key, &session.id).await.map_err(BridgeError::SessionError)?;
    let _ = app.emit("wakeup-completed", &session.id);
    Ok(session.id)
}
