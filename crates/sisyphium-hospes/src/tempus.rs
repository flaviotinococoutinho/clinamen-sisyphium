//! Conservative civil clock conversion and catch-up (REQ-H-06).
use std::time::{SystemTime, UNIX_EPOCH};

/// Converts a civil clock to signed epoch milliseconds (REQ-H-06).
/// Returns `None` when the timestamp does not fit in `i64`.
#[must_use]
pub fn epoch_ms(time: SystemTime) -> Option<i64> {
    let value = match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => i128::try_from(duration.as_millis()).ok()?,
        Err(error) => -i128::try_from(error.duration().as_millis()).ok()?,
    };
    i64::try_from(value).ok()
}

/// Counts complete elapsed ticks, capped at 65536 (REQ-H-06).
/// Missing or future timestamps and a zero period produce no evolution.
#[must_use]
pub fn elapsed_ticks(now: i64, mtime: Option<i64>, tick_ms: u32) -> u32 {
    let Some(previous) = mtime else { return 0 };
    if tick_ms == 0 {
        return 0;
    }
    ((i128::from(now) - i128::from(previous)).max(0) / i128::from(tick_ms)).min(65_536) as u32
}
