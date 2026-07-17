use base64::Engine;
use rand::{rngs::OsRng, RngCore};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

const TOKEN_BYTES: usize = 32;

pub fn generate_token() -> Zeroizing<String> {
    let mut bytes = Zeroizing::new([0_u8; TOKEN_BYTES]);
    OsRng.fill_bytes(bytes.as_mut());
    Zeroizing::new(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes.as_ref()))
}

pub fn valid_bearer(header: &str, expected: &str) -> bool {
    let Some(candidate) = header.strip_prefix("Bearer ") else {
        return false;
    };
    if candidate.len() != expected.len() {
        return false;
    }
    bool::from(candidate.as_bytes().ct_eq(expected.as_bytes()))
}
