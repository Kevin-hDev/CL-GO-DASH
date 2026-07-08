use super::*;

#[test]
fn reminder_is_due_immediately_then_after_interval() {
    let now = Instant::now();
    assert!(should_emit_reminder(false, None, now));
    assert!(!should_emit_reminder(true, Some(now), now));
    assert!(should_emit_reminder(
        true,
        Some(now - REMINDER_INTERVAL),
        now
    ));
}
