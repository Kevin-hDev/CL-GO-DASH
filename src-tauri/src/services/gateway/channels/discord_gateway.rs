use std::sync::Arc;
use std::time::Duration;

use futures_util::SinkExt;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use zeroize::Zeroizing;

use super::discord_types::*;

pub type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    WsMessage,
>;

pub async fn heartbeat_loop(
    sink: Arc<Mutex<WsSink>>,
    cancel: tokio_util::sync::CancellationToken,
    interval: Duration,
    seq: Option<u64>,
) {
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(interval) => {
                let hb = Heartbeat { op: 1, d: seq };
                let json = serde_json::to_string(&hb).unwrap_or_default();
                if sink.lock().await.send(WsMessage::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

pub fn build_identify(token: &Zeroizing<String>) -> Identify {
    Identify {
        op: 2,
        d: IdentifyData {
            token: token.as_str().to_string(),
            intents: INTENT_GUILDS | INTENT_GUILD_MESSAGES | INTENT_DM_MESSAGES | INTENT_MESSAGE_CONTENT,
            properties: IdentifyProperties {
                os: "linux".into(),
                browser: "cl-go-dash".into(),
                device: "cl-go-dash".into(),
            },
        },
    }
}
