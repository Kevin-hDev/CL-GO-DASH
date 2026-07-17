use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::oauth_providers::ProviderId;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

const MAX_CONNECTIONS: usize = 8;

pub(crate) struct ServerContext {
    pub provider: ProviderId,
    pub working_dir: PathBuf,
    pub session_id: String,
    pub mode: String,
    pub emitter: AgentEventEmitter,
    pub cancel: CancellationToken,
}

pub struct InternalMcpServer {
    address: std::net::SocketAddr,
    token: Arc<Zeroizing<String>>,
    task: tokio::task::JoinHandle<()>,
}

impl InternalMcpServer {
    pub async fn start(
        provider: ProviderId,
        working_dir: &Path,
        session_id: &str,
        mode: &str,
        emitter: AgentEventEmitter,
        cancel: CancellationToken,
    ) -> Result<Self, String> {
        if !working_dir.is_absolute() || !working_dir.is_dir() {
            return Err("Serveur MCP indisponible".to_string());
        }
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(|_| "Serveur MCP indisponible".to_string())?;
        let address = listener
            .local_addr()
            .map_err(|_| "Serveur MCP indisponible".to_string())?;
        let token = Arc::new(super::auth::generate_token());
        let context = Arc::new(ServerContext {
            provider,
            working_dir: working_dir.to_path_buf(),
            session_id: session_id.to_string(),
            mode: mode.to_string(),
            emitter,
            cancel,
        });
        let task_token = token.clone();
        let task = tokio::spawn(run(listener, context, task_token));
        Ok(Self {
            address,
            token,
            task,
        })
    }

    pub fn configuration(&self) -> Value {
        json!({
            "type": "http",
            "name": "cl-go",
            "url": format!("http://{}/mcp", self.address),
            "headers": [{
                "name": "Authorization",
                "value": format!("Bearer {}", self.token.as_str()),
            }],
        })
    }
}

impl Drop for InternalMcpServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn run(listener: TcpListener, context: Arc<ServerContext>, token: Arc<Zeroizing<String>>) {
    let permits = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    loop {
        let Ok((mut stream, peer)) = listener.accept().await else {
            break;
        };
        if !peer.ip().is_loopback() {
            continue;
        }
        let Ok(permit) = permits.clone().try_acquire_owned() else {
            continue;
        };
        let context = context.clone();
        let token = token.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let request = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                super::http::read_request(&mut stream),
            )
            .await;
            let Ok(Ok(request)) = request else {
                let _ = super::http::write_response(&mut stream, "400 Bad Request", None).await;
                return;
            };
            if !super::auth::valid_bearer(&request.authorization, token.as_str()) {
                let _ = super::http::write_response(&mut stream, "401 Unauthorized", None).await;
                return;
            }
            match super::rpc::handle(&context, &request.body).await {
                super::rpc::RpcResponse::Empty => {
                    let _ = super::http::write_response(&mut stream, "202 Accepted", None).await;
                }
                super::rpc::RpcResponse::Json(value) => {
                    let _ = super::http::write_response(&mut stream, "200 OK", Some(&value)).await;
                }
            }
        });
    }
}
