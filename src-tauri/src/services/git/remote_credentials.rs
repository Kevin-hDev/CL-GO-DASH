use git2::{Cred, CredentialType};

use super::repo;

pub(super) fn credentials(
    url: &str,
    username: Option<&str>,
    allowed: CredentialType,
    github_token: Option<&str>,
) -> Result<Cred, git2::Error> {
    if repo::is_github_url(url) && allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
        if let Some(token) = github_token {
            return Cred::userpass_plaintext("x-access-token", token);
        }
    }
    if allowed.contains(CredentialType::SSH_KEY) {
        return Cred::ssh_key_from_agent(username.unwrap_or("git"));
    }
    Cred::default()
}
