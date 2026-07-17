#![no_main]
//! Fuzz the latency / timeframe arithmetic: an arbitrary timeframe string is
//! parsed to milliseconds (malformed input is a clean `Err`, never a panic), and
//! the snapshot-index selection is exercised over bounded parameters. The index,
//! when returned, must always be a valid bar.

use impact_core::latency::{bar_ms, snapshot_index};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    // Timeframe parsing must never panic; a bad string is an Err.
    let bar = bar_ms(text).unwrap_or(3_600_000);

    // Derive bounded parameters from the input bytes and check the selection.
    let signal_bar = usize::from(data.first().copied().unwrap_or(0));
    let latency_ms = u64::from(data.get(1).copied().unwrap_or(0)) * 60_000;
    let n_bars = usize::from(data.get(2).copied().unwrap_or(0)).max(1);
    let next_open = data.get(3).copied().unwrap_or(0) & 1 == 1;

    if let Some(idx) = snapshot_index(signal_bar, latency_ms, bar, next_open, n_bars) {
        assert!(idx < n_bars, "a returned snapshot index must be a valid bar");
    }
});
