use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use zeroize::Zeroizing;

use super::{
    capabilities::ChannelCapabilities, ChannelAdapter, ChannelContext, DeliveryReceipt,
    GatewayError, GatewayResult, InboundMessage, OutboundMessage,
};
use super::slack_types::*;
use crate::services::api_keys;
use crate::services::gateway::types::ChannelKey;

pub struct SlackAdapter {
    client: Client,
    state: Arc<RwLock<SlackState>>,
}

struct SlackState {
    app_token: Option<Zeroizing<String>>,
    bot_token: Option<Zeroizing<String>>,
}

impl SlackAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::builder().timeout(Duration::from_secs(30)).build().expect("http client"),
            state: Arc::new(RwLock::new(SlackState {
                app_token: None,
                bot_token: None,
            })),
        }
    }

    async fn load_tokens(&self, vault_prefix: &str) -> GatewayResult<()> {
        let bot = api_keys::get_raw(&format!("{vault_prefix}.bot"))
            .map_err(|_| GatewayError::auth("token bot Slack manquant"))?;
        let app = api_keys::get_raw(&format!("{vault_prefix}.app"))
            .map_err(|_| GatewayError::auth("token app Slack manquant"))?;
        let mut s = self.state.write().await;
        s.bot_token = Some(bot);
        s.app_token = Some(app);
        Ok(())
    }

    async fn get_ws_url(client: &Client, app_token: &str) -> GatewayResult<String> {
        let resp: SlackSocketUrl = client
            .post("https://slack.com/api/apps.connections.open")
            .bearer_auth(app_token)
            .send().await.map_err(|e| GatewayError::network(format!("ws url: {e}")))?
            .json().await.map_err(|e| GatewayError::network(format!("parse: {e}")))?;
        resp.url.ok_or_else(|| GatewayError::auth(resp.error.unwrap_or_default()))
    }

    fn to_inbound(evt: &SlackEvent, key: &ChannelKey) -> Option<InboundMessage> {
        if !evt.is_user_message() { return None; }
        Some(InboundMessage {
            channel_key: key.clone(),
            user_id: evt.user.clone()?,
            content: evt.text.clone()?,
            message_id: evt.ts.clone()?,
            chat_id: evt.channel.clone()?,
            is_group: true,
            mentions_bot: false,
        })
    }
}

#[async_trait]
impl ChannelAdapter for SlackAdapter {
    fn id(&self) -> &'static str { "slack" }

    fn capabilities(&self) -> ChannelCapabilities { ChannelCapabilities::slack() }

    async fn validate_config(&self, cfg: &crate::models::ChannelAccountConfig) -> GatewayResult<()> {
        let prefix = format!("gateway.slack.{}", cfg.account_id);
        if !api_keys::has_key(&format!("raw:{prefix}.bot")) {
            return Err(GatewayError::auth("token bot Slack non configuré"));
        }
        if !api_keys::has_key(&format!("raw:{prefix}.app")) {
            return Err(GatewayError::auth("token app Slack non configuré"));
        }
        Ok(())
    }

    async fn start(
        &self, ctx: ChannelContext, sender: mpsc::Sender<InboundMessage>,
    ) -> GatewayResult<tokio::task::JoinHandle<()>> {
        self.load_tokens(&format!("gateway.slack.{}", ctx.key.account_id)).await?;
        let state = self.state.clone();
        let client = self.client.clone();
        let cancel = ctx.cancel;
        let key = ctx.key;

        Ok(tokio::spawn(async move {
            loop {
                if cancel.is_cancelled() { break; }
                let app_token = {
                    let s = state.read().await;
                    match &s.app_token { Some(t) => t.clone(), None => break }
                };
                let ws_url = match Self::get_ws_url(&client, &app_token).await {
                    Ok(u) => u, Err(_) => { tokio::time::sleep(Duration::from_secs(10)).await; continue; }
                };
                let ws = match tokio_tungstenite::connect_async(&ws_url).await {
                    Ok((s, _)) => s, Err(_) => { tokio::time::sleep(Duration::from_secs(5)).await; continue; }
                };
                let (mut sink, mut stream) = ws.split();
                loop {
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        msg = stream.next() => {
                            let Some(Ok(WsMessage::Text(txt))) = msg else { break; };
                            let Ok(sm) = serde_json::from_str::<SlackSocketMessage>(&txt) else { continue; };
                            if let Some(eid) = &sm.envelope_id {
                                let ack = serde_json::to_string(&SlackAck { envelope_id: eid.clone() }).unwrap_or_default();
                                let _ = sink.send(WsMessage::Text(ack.into())).await;
                            }
                            if let Some(payload) = &sm.payload {
                                if let Some(evt) = &payload.event {
                                    if let Some(inbound) = Self::to_inbound(evt, &key) {
                                        let _ = sender.send(inbound).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }))
    }

    async fn send(&self, msg: OutboundMessage) -> GatewayResult<DeliveryReceipt> {
        let token = {
            let s = self.state.read().await;
            s.bot_token.clone().ok_or_else(|| GatewayError::auth("pas de token bot"))?
        };
        let body = serde_json::json!({
            "channel": msg.chat_id,
            "text": msg.content,
        });
        let resp: SlackPostResponse = self.client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(token.as_str())
            .json(&body)
            .send().await.map_err(|e| GatewayError::network(format!("send: {e}")))?
            .json().await.map_err(|e| GatewayError::network(format!("parse: {e}")))?;

        match resp.ts {
            Some(ts) => Ok(DeliveryReceipt { message_id: ts }),
            None => Err(GatewayError::network(resp.error.unwrap_or_default())),
        }
    }

    async fn health(&self) -> bool {
        self.state.read().await.bot_token.is_some()
    }
}
