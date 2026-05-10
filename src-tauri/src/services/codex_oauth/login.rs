use rand::RngCore;
use zeroize::Zeroizing;

use super::{callback, jwt, pkce, store, token};

use super::{CLIENT_ID, REDIRECT_URI};

const AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const SCOPES: &str = "openid profile email offline_access";

fn generate_state() -> Zeroizing<String> {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let s = hex::encode(bytes);
    bytes.fill(0);
    Zeroizing::new(s)
}

fn build_auth_url(challenge: &str, state: &str) -> String {
    format!(
        "{AUTH_URL}?response_type=code\
         &client_id={CLIENT_ID}\
         &redirect_uri={}\
         &scope={}\
         &code_challenge={challenge}\
         &code_challenge_method=S256\
         &state={state}",
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(SCOPES),
    )
}

pub async fn login() -> Result<String, String> {
    let pair = pkce::generate();
    let state = generate_state();
    let url = build_auth_url(&pair.challenge, &state);

    open::that(&url).map_err(|e| format!("impossible d'ouvrir le navigateur: {e}"))?;
    eprintln!("[codex] navigateur ouvert, attente du callback...");

    let cb = callback::wait_for_callback(&state).await?;
    eprintln!("[codex] code reçu, échange en cours...");

    let creds = token::exchange_code(&cb.code, pair.verifier.as_str()).await?;
    let email = jwt::extract_claims(&creds.access)
        .ok()
        .and_then(|c| c.email)
        .unwrap_or_else(|| "inconnu".to_string());

    store::save(&creds)?;
    eprintln!("[codex] connecté");

    Ok(email)
}

pub fn logout() -> Result<(), String> {
    store::clear()
}
