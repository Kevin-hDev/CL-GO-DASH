use super::timeouts::{compression_timeout, COMPRESSION_TIMEOUT_SECS};

#[test]
fn compression_timeout_is_ten_minutes() {
    assert_eq!(COMPRESSION_TIMEOUT_SECS, 600);
    assert_eq!(compression_timeout().as_secs(), 600);
}
