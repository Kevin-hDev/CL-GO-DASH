use crate::services::oauth_providers::ProviderId;

pub fn auth_method(provider: ProviderId, methods: &serde_json::Value) -> Option<&str> {
    let available = methods.as_array()?;
    let wanted = match provider {
        ProviderId::Moonshot => ["login", ""],
        ProviderId::Xai => ["cached_token", ""],
        ProviderId::OpenAi => return None,
    };
    wanted
        .into_iter()
        .filter(|id| !id.is_empty())
        .find(|wanted_id| {
            available
                .iter()
                .any(|method| method["id"].as_str() == Some(*wanted_id))
        })
}

pub fn native_tool_allowed(provider: ProviderId, tool_name: &str) -> bool {
    provider != ProviderId::Xai || !tool_name.eq_ignore_ascii_case("bash")
}
