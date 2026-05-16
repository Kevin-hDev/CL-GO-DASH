use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use zeroize::Zeroizing;

use super::discord_gateway::{build_identify, heartbeat_loop};
use super::discord_types::*;
use super::{
    capabilities::ChannelCapabilities, ChannelAdapter, ChannelContext, DeliveryReceipt,
    GatewayError, GatewayResult, InboundMessage, OutboundMessage,
};
use crate::services::gateway::tokens;

pub struct DiscordAdapter {
    pub(super) client: Client,
    pub(super) state: Arc<RwLock<DiscordState>>,
}

pub(super) struct DiscordState {
    pub(super) bot_token: Option<Zeroizing<String>>,
    pub(super) bot_user_id: String,
}

impl DiscordAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("http client"),
            state: Arc::new(RwLock::new(DiscordState {
                bot_token: None,
                bot_user_id: String::new(),
            })),
        }
    }

}

#[async_trait]
impl ChannelAdapter for DiscordAdapter {
    fn id(&self) -> &'static str {
        "discord"
    }
    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities::discord()
    }

    async fn validate_config(
        &self,
        cfg: &crate::models::ChannelAccountConfig,
    ) -> GatewayResult<()> {
        if !tokens::has("discord", &cfg.account_id, "default").unwrap_or(false) {
            return Err(GatewayError::auth("token Discord non configuré"));
        }
        Ok(())
    }

    async fn start(
        &self,
        ctx: ChannelContext,
        sender: mpsc::Sender<InboundMessage>,
    ) -> GatewayResult<tokio::task::JoinHandle<()>> {
        self.load_token(&ctx.key.vault_key()).await?;
        let state = self.state.clone();
        let cancel = ctx.cancel;
        let key = ctx.key;
        let require_mention = ctx.config.require_mention;

        Ok(tokio::spawn(async move {
            loop {
                if cancel.is_cancelled() {
                    break;
                }
                let token = {
                    let s = state.read().await;
                    match &s.bot_token {
                        Some(t) => t.clone(),
                        None => break,
                    }
                };
                let ws = match tokio_tungstenite::connect_async(GATEWAY_URL).await {
                    Ok((s, _)) => s,
                    Err(_) => {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };
                let (sink, mut stream) = ws.split();
                let shared_sink = Arc::new(Mutex::new(sink));
                let mut seq: Option<u64> = None;

                while let Some(Ok(WsMessage::Text(txt))) = stream.next().await {
                    if cancel.is_cancelled() {
                        break;
                    }
                    let Ok(payload) = serde_json::from_str::<GatewayPayload>(&txt) else {
                        continue;
                    };
                    if let Some(s) = payload.s {
                        seq = Some(s);
                    }
                    match payload.op {
                        10 => {
                            if let Some(d) = &payload.d {
                                if let Ok(hello) = serde_json::from_value::<GatewayHello>(d.clone())
                                {
                                    let interval = Duration::from_millis(hello.heartbeat_interval);
                                    tokio::spawn(heartbeat_loop(
                                        shared_sink.clone(),
                                        cancel.clone(),
                                        interval,
                                        seq,
                                    ));
                                }
                            }
                            let json =
                                serde_json::to_string(&build_identify(&token)).unwrap_or_default();
                            let _ = shared_sink
                                .lock()
                                .await
                                .send(WsMessage::Text(json.into()))
                                .await;
                        }
                        0 if payload.t.as_deref() == Some("READY") => {
                            if let Some(d) = &payload.d {
                                if let Ok(ready) = serde_json::from_value::<ReadyEvent>(d.clone()) {
                                    state.write().await.bot_user_id = ready.user.id;
                                }
                            }
                        }
                        0 if payload.t.as_deref() == Some("MESSAGE_CREATE") => {
                            if let Some(d) = payload.d {
                                if let Ok(msg) = serde_json::from_value::<DiscordMessage>(d) {
                                    let bot_id = state.read().await.bot_user_id.clone();
                                    if let Some(inbound) =
                                        Self::to_inbound(&msg, &key, require_mention, &bot_id)
                                    {
                                        let _ = sender.send(inbound).await;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if cancel.is_cancelled() {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }))
    }

    async fn send(&self, msg: OutboundMessage) -> GatewayResult<DeliveryReceipt> {
        let token = {
            let s = self.state.read().await;
            s.bot_token
                .clone()
                .ok_or_else(|| GatewayError::auth("pas de token"))?
        };
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages",
            msg.chat_id
        );
        let body = SendMessage {
            content: msg.content,
            allowed_mentions: AllowedMentions { parse: vec![] },
            message_reference: msg.reply_to.map(|id| MessageReference { message_id: id }),
        };
        let resp: SentMessage = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", token.as_str()))
            .json(&body)
            .send()
            .await
            .map_err(|e| GatewayError::network(format!("send: {e}")))?
            .json()
            .await
            .map_err(|e| GatewayError::network(format!("parse: {e}")))?;
        Ok(DeliveryReceipt {
            message_id: resp.id,
        })
    }

    async fn health(&self) -> bool {
        self.state.read().await.bot_token.is_some()
    }
}
