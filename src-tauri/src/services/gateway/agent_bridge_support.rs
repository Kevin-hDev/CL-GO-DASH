use tauri::Emitter;

use super::agent_bridge::BridgeError;
use super::channels::InboundMessage;
use super::security::{
    audit::{self, AuditAction},
    ids, validation,
};
use super::session_map;
use crate::models::{ChannelAccountConfig, GatewayConfig};
use crate::services::agent_local::session_store;

pub(crate) fn resolve_provider_model(
    account: &ChannelAccountConfig,
    config: &GatewayConfig,
) -> (String, String) {
    if !account.provider.is_empty() && !account.model.is_empty() {
        return (account.provider.clone(), account.model.clone());
    }
    (
        config.default_provider.clone(),
        config.default_model.clone(),
    )
}

pub(crate) fn validate_inbound(msg: &InboundMessage) -> Result<(), BridgeError> {
    ids::validate_channel_id(&msg.channel_key.channel_id).map_err(BridgeError::Blocked)?;
    ids::validate_account_id(&msg.channel_key.account_id).map_err(BridgeError::Blocked)?;
    ids::validate_external_id(&msg.user_id).map_err(BridgeError::Blocked)?;
    ids::validate_external_id(&msg.message_id).map_err(BridgeError::Blocked)?;
    ids::validate_external_id(&msg.chat_id).map_err(BridgeError::Blocked)?;
    let vr = validation::validate_message(&msg.content);
    if !vr.valid {
        return Err(BridgeError::Blocked(vr.reason.unwrap_or_default()));
    }
    Ok(())
}

pub(crate) fn block(msg: &InboundMessage, reason: &str) -> BridgeError {
    audit_msg(msg, AuditAction::Blocked, Some(reason), None);
    BridgeError::Blocked(reason.to_string())
}

pub(crate) fn audit_msg(
    msg: &InboundMessage,
    action: AuditAction,
    decision: Option<&str>,
    error: Option<&str>,
) {
    audit::log_gateway_action(
        &msg.channel_key.channel_id,
        &msg.channel_key.account_id,
        &msg.user_id,
        action,
        decision,
        error,
    );
}

pub(crate) fn emit_session_updated(app: &tauri::AppHandle, session_id: &str) {
    let payload = serde_json::json!({ "session_id": session_id });
    let _ = app.emit("agent-session-updated", &payload);
    let _ = app.emit("wakeup-completed", &payload);
}

pub(crate) async fn sync_session_model(session_id: &str, provider: &str, model: &str) {
    if let Ok(session) = session_store::get(session_id).await {
        if session.provider != provider || session.model != model {
            let _ = session_store::update_model(session_id, model, provider, None, None).await;
        }
    }
}

pub(crate) fn find_account_config(
    config: &GatewayConfig,
    msg: &InboundMessage,
) -> Option<ChannelAccountConfig> {
    let accounts = match msg.channel_key.channel_id.as_str() {
        "telegram" => &config.channels.telegram,
        "slack" => &config.channels.slack,
        "discord" => &config.channels.discord,
        _ => return None,
    };
    accounts
        .iter()
        .find(|a| a.account_id == msg.channel_key.account_id)
        .cloned()
}

pub(crate) fn build_external_key(msg: &InboundMessage) -> String {
    format!(
        "{}/{}/{}",
        msg.channel_key.channel_id, msg.channel_key.account_id, msg.user_id
    )
}

pub(crate) async fn find_or_create_session(
    msg: &InboundMessage,
    channel_key: &str,
    provider: &str,
    model: &str,
    max_sessions: usize,
    app: &tauri::AppHandle,
) -> Result<String, BridgeError> {
    if let Some(id) = session_map::find(channel_key).await {
        if session_store::get(&id).await.is_ok() {
            return Ok(id);
        }
    }
    let label = match msg.channel_key.channel_id.as_str() {
        "telegram" => "TG",
        "slack" => "SL",
        "discord" => "DC",
        _ => "GW",
    };
    let name = format!("[{label}] {}", msg.user_id);
    let session = session_store::create_gateway(&name, model, provider, channel_key.to_string())
        .await
        .map_err(BridgeError::SessionError)?;
    session_map::insert_bounded(channel_key, &session.id, max_sessions)
        .await
        .map_err(BridgeError::SessionError)?;
    emit_session_updated(app, &session.id);
    Ok(session.id)
}
