pub mod types;
pub mod storage;
pub mod pkce;
pub mod discovery;
pub mod callback_server;
pub mod flow;
pub mod flow_auth;

const MAX_OAUTH_BODY: usize = 512 * 1024;

pub(crate) async fn bounded_json<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, String> {
    let bytes = resp.bytes().await.map_err(|_| "réponse illisible".to_string())?;
    if bytes.len() > MAX_OAUTH_BODY {
        return Err("réponse OAuth trop volumineuse".to_string());
    }
    serde_json::from_slice(&bytes).map_err(|_| "réponse invalide".to_string())
}
