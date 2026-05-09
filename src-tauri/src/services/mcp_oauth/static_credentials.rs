pub struct StaticCredentials {
    pub client_id: &'static str,
    pub client_secret: &'static str,
    pub scopes: &'static str,
}

pub fn for_endpoint(endpoint: &str) -> Option<StaticCredentials> {
    let host = reqwest::Url::parse(endpoint).ok()?.host_str()?.to_string();

    if host.ends_with(".googleapis.com") {
        let scopes = google_scopes_for_host(&host);
        return Some(StaticCredentials {
            client_id: env!("CLGO_GOOGLE_CLIENT_ID"),
            client_secret: env!("CLGO_GOOGLE_CLIENT_SECRET"),
            scopes,
        });
    }

    if host == "api.githubcopilot.com" || host.ends_with(".githubcopilot.com") {
        return Some(StaticCredentials {
            client_id: env!("CLGO_GITHUB_CLIENT_ID"),
            client_secret: env!("CLGO_GITHUB_CLIENT_SECRET"),
            scopes: "repo read:user",
        });
    }

    None
}

fn google_scopes_for_host(host: &str) -> &'static str {
    if host.starts_with("gmail") {
        return "https://www.googleapis.com/auth/gmail.modify";
    }
    if host.starts_with("drive") {
        return "https://www.googleapis.com/auth/drive";
    }
    if host.starts_with("calendar") {
        return "https://www.googleapis.com/auth/calendar";
    }
    "https://www.googleapis.com/auth/userinfo.email"
}
