pub mod callback_server;
pub mod discovery;
pub mod flow;
pub mod flow_auth;
pub mod pkce;
pub mod static_credentials;
pub mod storage;
pub mod trusted_oauth;
pub mod types;

const MAX_OAUTH_BODY: usize = 512 * 1024;

pub(crate) async fn bounded_json<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, String> {
    let mut bytes = resp
        .bytes()
        .await
        .map_err(|_| "réponse illisible".to_string())?
        .to_vec();
    if bytes.len() > MAX_OAUTH_BODY {
        zeroize::Zeroize::zeroize(&mut bytes);
        return Err("réponse OAuth trop volumineuse".to_string());
    }
    let result = serde_json::from_slice(&bytes).map_err(|_| "réponse invalide".to_string());
    zeroize::Zeroize::zeroize(&mut bytes);
    result
}
