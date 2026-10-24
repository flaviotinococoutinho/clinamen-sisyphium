//! Conservative clock contract (REQ-H-06).
use sisyphium_hospes::tempus::{elapsed_ticks, epoch_ms};
use std::time::{Duration, UNIX_EPOCH};

/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn elapsed_ticks_conservative() {
    assert_eq!(elapsed_ticks(1000, Some(958), 42), 1);
    assert_eq!(elapsed_ticks(1000, Some(1001), 42), 0);
    assert_eq!(elapsed_ticks(1000, None, 42), 0);
    assert_eq!(elapsed_ticks(i64::MAX, Some(i64::MIN), 42), 65_536);
    assert_eq!(elapsed_ticks(1000, Some(0), 0), 0);
    assert_eq!(elapsed_ticks(-1000, Some(-1042), 42), 1);
    assert_eq!(epoch_ms(UNIX_EPOCH), Some(0));
    assert_eq!(
        epoch_ms(UNIX_EPOCH - Duration::from_millis(1001)),
        Some(-1001)
    );
    assert_eq!(
        epoch_ms(UNIX_EPOCH + Duration::from_millis(1001)),
        Some(1001)
    );
}

/// Timestamp conversion checks both signed limits where the platform represents them (REQ-H-06).
/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn epoch_range_is_checked() {
    for (milliseconds, expected) in [
        (i64::MAX as u64, Some(i64::MAX)),
        (i64::MAX as u64 + 1, None),
    ] {
        if let Some(time) = UNIX_EPOCH.checked_add(Duration::from_millis(milliseconds)) {
            assert_eq!(epoch_ms(time), expected);
        }
    }
    if let Some(time) = UNIX_EPOCH.checked_sub(Duration::from_millis(i64::MAX as u64 + 1)) {
        assert_eq!(epoch_ms(time), Some(i64::MIN));
    }
    if let Some(time) = UNIX_EPOCH.checked_sub(Duration::from_millis(i64::MAX as u64 + 2)) {
        assert_eq!(epoch_ms(time), None);
    }
}
