//! Timeframe parsing and latency-to-snapshot mapping. Pure integer arithmetic, so
//! the choice of which book snapshot an order fills against is deterministic.

use crate::error::{Error, Result};

/// Milliseconds per bar for a timeframe string such as `"1h"`, `"5m"`, `"30s"`,
/// `"1d"` or `"1w"`. The suffix is the unit; the prefix is a positive integer.
///
/// # Errors
///
/// Returns [`Error::BadSpec`] if the timeframe has no unit, a non-numeric or zero
/// count, or an unknown unit suffix.
pub fn bar_ms(timeframe: &str) -> Result<u64> {
    let tf = timeframe.trim();
    let (num, unit) = tf.split_at(
        tf.find(|c: char| !c.is_ascii_digit())
            .ok_or_else(|| Error::BadSpec(format!("timeframe missing unit: {tf}")))?,
    );
    let n: u64 = num
        .parse()
        .map_err(|_| Error::BadSpec(format!("timeframe bad count: {tf}")))?;
    if n == 0 {
        return Err(Error::BadSpec(format!("timeframe count must be > 0: {tf}")));
    }
    let unit_ms: u64 = match unit {
        "s" => 1_000,
        "m" => 60_000,
        "h" => 3_600_000,
        "d" => 86_400_000,
        "w" => 604_800_000,
        _ => return Err(Error::BadSpec(format!("timeframe bad unit: {tf}"))),
    };
    Ok(n * unit_ms)
}

/// The bar index whose order book a signalled order fills against. `next_open`
/// adds one bar (the look-ahead-free default); latency adds `ceil(latency_ms /
/// bar_ms)` bars on top. Returns `None` if the target runs past `n_bars` (the
/// order is cancelled).
#[must_use]
pub fn snapshot_index(
    signal_bar: usize,
    latency_ms: u64,
    bar_ms: u64,
    next_open: bool,
    n_bars: usize,
) -> Option<usize> {
    let base = signal_bar + usize::from(next_open);
    let extra = if bar_ms == 0 {
        0
    } else {
        // ceil(latency_ms / bar_ms)
        usize::try_from(latency_ms.div_ceil(bar_ms)).unwrap_or(usize::MAX)
    };
    let idx = base.saturating_add(extra);
    (idx < n_bars).then_some(idx)
}

#[cfg(test)]
mod tests {
    use super::{bar_ms, snapshot_index};

    #[test]
    fn parses_common_timeframes() {
        assert_eq!(bar_ms("1h").unwrap(), 3_600_000);
        assert_eq!(bar_ms("5m").unwrap(), 300_000);
        assert_eq!(bar_ms("30s").unwrap(), 30_000);
        assert_eq!(bar_ms("1d").unwrap(), 86_400_000);
        assert_eq!(bar_ms("1w").unwrap(), 604_800_000);
    }

    #[test]
    fn rejects_bad_timeframes() {
        assert!(bar_ms("1x").is_err());
        assert!(bar_ms("h").is_err());
        assert!(bar_ms("0h").is_err());
        assert!(bar_ms("abc").is_err());
    }

    #[test]
    fn zero_latency_next_open_is_next_bar() {
        assert_eq!(snapshot_index(0, 0, 3_600_000, true, 5), Some(1));
        assert_eq!(snapshot_index(0, 0, 3_600_000, false, 5), Some(0));
    }

    #[test]
    fn latency_shifts_forward_and_can_cancel() {
        // 1h bars, 90 min latency -> ceil(90/60) = 2 extra bars, + next_open.
        assert_eq!(snapshot_index(0, 5_400_000, 3_600_000, true, 5), Some(3));
        // runs off the end -> cancelled.
        assert_eq!(snapshot_index(4, 5_400_000, 3_600_000, true, 5), None);
    }
}
