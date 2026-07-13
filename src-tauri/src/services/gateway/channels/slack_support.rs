use reqwest::Client;
use zeroize::Zeroizing;

use super::slack::SlackAdapter;
use super::slack_types::*;
use super::{GatewayError, GatewayResult, InboundMessage};
use crate::services::api_keys;
use crate::services::gateway::types::ChannelKey;

impl SlackAdapter {
    pub(super) async fn load_tokens(&self, vault_prefix: &str) -> GatewayResult<()> {
        let bot = api_keys::get_raw(&format!("{vault_prefix}.bot"))
            .map_err(|_| GatewayError::auth("token bot Slack manquant"))?;
        let app = api_keys::get_raw(&format!("{vault_prefix}.app"))
            .map_err(|_| GatewayError::auth("token app Slack manquant"))?;
        let mut s = self.state.write().await;
        s.bot_token = Some(bot);
        s.app_token = Some(app);
        Ok(())
    }

    pub(super) async fn get_ws_url(
        client: &Client,
        app_token: &str,
    ) -> GatewayResult<Zeroizing<String>> {
        let resp: SlackSocketUrl = client
            .post("https://slack.com/api/apps.connections.open")
            .bearer_auth(app_token)
            .send()
            .await
            .map_err(|e| GatewayError::network(format!("ws url: {e}")))?
            .json()
            .await
            .map_err(|e| GatewayError::network(format!("parse: {e}")))?;
        if !resp.ok {
            return Err(GatewayError::auth(resp.error.unwrap_or_default()));
        }
        resp.url
            .map(Zeroizing::new)
            .ok_or_else(|| GatewayError::auth("url websocket Slack manquante"))
    }

    pub(super) fn to_inbound(evt: &SlackEvent, key: &ChannelKey) -> Option<InboundMessage> {
        if !evt.is_user_message() {
            return None;
        }
        Some(InboundMessage {
            channel_key: key.clone(),
            user_id: evt.user.clone()?,
            content: evt.text.clone()?,
            message_id: evt.thread_ts.clone().or_else(|| evt.ts.clone())?,
            chat_id: evt.channel.clone()?,
            thread_id: evt.thread_ts.clone(),
            is_group: true,
            mentions_bot: false,
        })
    }
}
