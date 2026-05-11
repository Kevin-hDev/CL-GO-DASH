use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::{mpsc, RwLock};
use zeroize::Zeroizing;

use super::telegram_types::*;
use super::{
    capabilities::ChannelCapabilities, ChannelAdapter, ChannelContext, DeliveryReceipt,
    GatewayError, GatewayResult, InboundMessage, OutboundMessage,
};
use crate::services::api_keys;

pub struct TelegramAdapter {
    client: Client,
    state: Arc<RwLock<TelegramState>>,
}

struct TelegramState {
    bot_token: Option<Zeroizing<String>>,
    bot_username: String,
    last_offset: i64,
}

impl TelegramAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(35))
                .build()
                .expect("http client"),
            state: Arc::new(RwLock::new(TelegramState {
                bot_token: None,
                bot_username: String::new(),
                last_offset: 0,
            })),
        }
    }

    pub fn api_url(token: &str, method: &str) -> String {
        format!("https://api.telegram.org/bot{token}/{method}")
    }

    async fn load_token_and_identity(&self, vault_key: &str) -> GatewayResult<()> {
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

        let bot_user = resp
            .result
            .ok_or_else(|| GatewayError::auth("getMe échoué"))?;
        let username = bot_user.username.unwrap_or_default();

        let mut state = self.state.write().await;
        state.bot_token = Some(token);
        state.bot_username = username;
        Ok(())
    }

    pub async fn poll_updates(
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
            return Err(GatewayError::network("conflit 409 — un autre poller actif"));
        }

        let body: TgResponse<Vec<TgUpdate>> = resp
            .json()
            .await
            .map_err(|e| GatewayError::network(format!("parse: {e}")))?;

        let updates = body.result.unwrap_or_default();
        if let Some(last) = updates.last() {
            state.write().await.last_offset = last.update_id;
        }
        Ok(updates)
    }

    pub fn to_inbound(
        update: &TgUpdate,
        channel_key: &crate::services::gateway::types::ChannelKey,
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

#[async_trait]
impl ChannelAdapter for TelegramAdapter {
    fn id(&self) -> &'static str {
        "telegram"
    }

    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities::telegram()
    }

    async fn validate_config(
        &self,
        cfg: &crate::models::ChannelAccountConfig,
    ) -> GatewayResult<()> {
        let vault_key = format!("gateway.telegram.{}", cfg.account_id);
        if !api_keys::has_key(&format!("raw:{vault_key}")) {
            return Err(GatewayError::auth("token Telegram non configuré"));
        }
        Ok(())
    }

    async fn start(
        &self,
        ctx: ChannelContext,
        sender: mpsc::Sender<InboundMessage>,
    ) -> GatewayResult<tokio::task::JoinHandle<()>> {
        self.load_token_and_identity(&ctx.key.vault_key()).await?;
        let client = self.client.clone();
        let state = self.state.clone();
        let cancel = ctx.cancel;
        let require_mention = ctx.config.require_mention;
        let channel_key = ctx.key;

        Ok(tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    result = Self::poll_updates(&client, &state) => {
                        match result {
                            Ok(updates) => {
                                let bot_name = state.read().await.bot_username.clone();
                                for u in updates {
                                    if let Some(m) = Self::to_inbound(&u, &channel_key, require_mention, &bot_name) {
                                        let _ = sender.send(m).await;
                                    }
                                }
                            }
                            Err(e) if e.is_auth => break,
                            Err(_) => tokio::time::sleep(Duration::from_secs(5)).await,
                        }
                    }
                }
            }
        }))
    }

    async fn send(&self, msg: OutboundMessage) -> GatewayResult<DeliveryReceipt> {
        let state = self.state.read().await;
        let token = state
            .bot_token
            .as_ref()
            .ok_or_else(|| GatewayError::auth("pas de token"))?;
        let url = Self::api_url(token, "sendMessage");

        let mut body = serde_json::json!({ "chat_id": msg.chat_id, "text": msg.content });
        if let Some(r) = &msg.reply_to {
            body["reply_to_message_id"] = serde_json::Value::String(r.clone());
        }

        let resp: TgResponse<TgSentMessage> = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| GatewayError::network(format!("send: {e}")))?
            .json()
            .await
            .map_err(|e| GatewayError::network(format!("parse: {e}")))?;

        match resp.result {
            Some(s) => Ok(DeliveryReceipt {
                message_id: s.message_id.to_string(),
            }),
            None => Err(GatewayError::network(resp.description.unwrap_or_default())),
        }
    }

    async fn health(&self) -> bool {
        self.state.read().await.bot_token.is_some()
    }
}
