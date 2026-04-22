use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_keys;
use crate::services::search;

pub async fn web_search(query: &str) -> Result<Vec<SearchResult>, String> {
    search::run_search(query).await
}

pub fn set_brave_key(key: &str) -> Result<(), String> {
    api_keys::set_key("brave", key)
}
