use std::time::Duration;

use reqwest::redirect::Policy;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use zeroize::Zeroizing;

use super::tokens::AccountTokens;

const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) struct ProbeEndpoints {
    telegram: String,
    slack: String,
    discord: String,
}

impl ProbeEndpoints {
    pub(crate) fn production() -> Self {
        Self {
            telegram: "https://api.telegram.org".into(),
            slack: "https://slack.com/api".into(),
            discord: "https://discord.com/api/v10".into(),
        }
    }

    #[cfg(test)]
    fn all(base: String) -> Self {
        Self {
            telegram: base.clone(),
            slack: base.clone(),
            discord: base,
        }
    }
}

pub(crate) async fn validate_tokens(
    channel_id: &str,
    credentials: &AccountTokens,
    endpoints: &ProbeEndpoints,
) -> Result<(), String> {
    credentials.validate_for(channel_id)?;
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .timeout(PROBE_TIMEOUT)
        .build()
        .map_err(|_| generic_error())?;
    match channel_id {
        "telegram" => validate_telegram(&client, credentials, endpoints).await,
        "slack" => validate_slack(&client, credentials, endpoints).await,
        "discord" => validate_discord(&client, credentials, endpoints).await,
        _ => Err(generic_error()),
    }
}

async fn validate_telegram(
    client: &reqwest::Client,
    credentials: &AccountTokens,
    endpoints: &ProbeEndpoints,
) -> Result<(), String> {
    let token = credentials.token().ok_or_else(generic_error)?;
    let url = Zeroizing::new(format!("{}/bot{token}/getMe", endpoints.telegram));
    let body: TelegramResponse = checked_json(client.get(url.as_str()).send().await).await?;
    if body.ok && body.result.is_some_and(|user| !user.id.is_empty()) {
        Ok(())
    } else {
        Err(generic_error())
    }
}

async fn validate_slack(
    client: &reqwest::Client,
    credentials: &AccountTokens,
    endpoints: &ProbeEndpoints,
) -> Result<(), String> {
    let bot: SlackResponse = checked_json(
        client
            .post(format!("{}/auth.test", endpoints.slack))
            .bearer_auth(credentials.bot_token().ok_or_else(generic_error)?)
            .send()
            .await,
    )
    .await?;
    if !bot.ok || bot.user_id.is_none_or(|id| id.is_empty()) {
        return Err(generic_error());
    }
    let app: SlackResponse = checked_json(
        client
            .post(format!("{}/apps.connections.open", endpoints.slack))
            .bearer_auth(credentials.app_token().ok_or_else(generic_error)?)
            .send()
            .await,
    )
    .await?;
    if !app.ok || app.url.is_none_or(|url| !url.starts_with("wss://")) {
        return Err(generic_error());
    }
    Ok(())
}

async fn validate_discord(
    client: &reqwest::Client,
    credentials: &AccountTokens,
    endpoints: &ProbeEndpoints,
) -> Result<(), String> {
    let auth = Zeroizing::new(format!(
        "Bot {}",
        credentials.token().ok_or_else(generic_error)?
    ));
    let body: DiscordResponse = checked_json(
        client
            .get(format!("{}/users/@me", endpoints.discord))
            .header("Authorization", auth.as_str())
            .send()
            .await,
    )
    .await?;
    if body.id.is_empty() {
        Err(generic_error())
    } else {
        Ok(())
    }
}

async fn checked_json<T: DeserializeOwned>(
    response: Result<reqwest::Response, reqwest::Error>,
) -> Result<T, String> {
    let response = response.map_err(|_| generic_error())?;
    if !response.status().is_success() {
        return Err(generic_error());
    }
    response.json().await.map_err(|_| generic_error())
}

fn generic_error() -> String {
    "validation des identifiants impossible".to_string()
}

#[derive(Deserialize)]
struct TelegramResponse {
    ok: bool,
    result: Option<Identity>,
}

#[derive(Deserialize)]
struct Identity {
    id: String,
}

#[derive(Deserialize)]
struct SlackResponse {
    ok: bool,
    user_id: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct DiscordResponse {
    id: String,
}

#[cfg(test)]
#[path = "token_probe_tests.rs"]
mod tests;
