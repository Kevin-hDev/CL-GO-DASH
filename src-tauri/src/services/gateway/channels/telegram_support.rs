use std::sync::Arc;

use reqwest::Client;
use tokio::sync::RwLock;

use super::telegram::{TelegramAdapter, TelegramState};
use super::telegram_types::*;
use super::{GatewayError, GatewayResult, InboundMessage};
use crate::services::api_keys;
use crate::services::gateway::types::ChannelKey;

impl TelegramAdapter {
    pub(super) async fn load_token_and_identity(&self, vault_key: &str) -> GatewayResult<()> {
        let token = api_keys::get_raw(vault_key)
            .map_err(|_| GatewayError::auth("token Telegram manquant dans le vault"))?;
        let url = Self::api_url(&token, "getMe");
        let resp: TgResponse<TgUser> = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GatewayError::network(format!("getMe: {e}")))?
            .json()
            .await
            .map_err(|e| GatewayError::network(format!("getMe parse: {e}")))?;
        if !resp.ok {
            return Err(GatewayError::auth(resp.error_message()));
        }
        let bot_user = resp
            .result
            .ok_or_else(|| GatewayError::auth("getMe échoué"))?;
        let mut state = self.state.write().await;
        state.bot_token = Some(token);
        state.bot_username = bot_user.username.unwrap_or_default();
        Ok(())
    }

    pub(super) async fn poll_updates(
        client: &Client,
        state: &Arc<RwLock<TelegramState>>,
    ) -> GatewayResult<Vec<TgUpdate>> {
        let (url, offset) = {
            let s = state.read().await;
            let token = s
                .bot_token
                .as_ref()
                .ok_or_else(|| GatewayError::auth("pas de token"))?;
            (Self::api_url(token, "getUpdates"), s.last_offset)
        };
        let resp = client
            .get(&url)
            .query(&[("offset", offset + 1), ("timeout", 30)])
            .send()
            .await
            .map_err(|e| GatewayError::network(format!("polling: {e}")))?;
        let status = resp.status().as_u16();
        if status == 401 || status == 403 {
            return Err(GatewayError::auth("token Telegram invalide"));
        }
        if status == 409 {
            return Err(GatewayError::network("conflit 409 - un autre poller actif"));
        }
        let body: TgResponse<Vec<TgUpdate>> = resp
            .json()
            .await
            .map_err(|e| GatewayError::network(format!("parse: {e}")))?;
        if !body.ok {
            return Err(GatewayError::network(body.error_message()));
        }
        let updates = body.result.unwrap_or_default();
        if let Some(last) = updates.last() {
            state.write().await.last_offset = last.update_id;
        }
        Ok(updates)
    }

    pub(super) fn to_inbound(
        update: &TgUpdate,
        channel_key: &ChannelKey,
        require_mention: bool,
        bot_username: &str,
    ) -> Option<InboundMessage> {
        let msg = update.message.as_ref()?;
        let text = msg.text.as_ref()?;
        let from = msg.from.as_ref()?;
        if msg.chat.is_group() && require_mention && !msg.has_bot_mention(bot_username) {
            return None;
        }
        Some(InboundMessage {
            channel_key: channel_key.clone(),
            user_id: from.id.to_string(),
            content: text.clone(),
            message_id: msg.message_id.to_string(),
            chat_id: msg.chat.id.to_string(),
            is_group: msg.chat.is_group(),
            mentions_bot: msg.has_bot_mention(bot_username),
        })
    }
}
