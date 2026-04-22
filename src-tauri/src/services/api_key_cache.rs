use zeroize::Zeroizing;

use super::api_keys;

pub fn get_key(provider_id: &str) -> Result<Zeroizing<String>, String> {
    api_keys::get_key(provider_id)
}

pub fn invalidate(_provider_id: &str) {}
