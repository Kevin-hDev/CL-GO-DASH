use std::time::Duration;

pub const COMPRESSION_TIMEOUT_SECS: u64 = 600;

pub fn compression_timeout() -> Duration {
    Duration::from_secs(COMPRESSION_TIMEOUT_SECS)
}
