//! Wickra Impact core — a market-impact backtester that walks the real
//! historical L2 order book so slippage is measured, not guessed.
//!
//! Scaffold. IMPACT inherits the `wickra-backtest` engine (its `StrategySpec`,
//! `RunRequest` and `BacktestReport`) and replaces only the fill stage with an
//! order-book-walk fill engine; that engine and the JSON command boundary
//! (`command_json`) land in P-IMP-1. This file pins the crate and its coupling
//! to the backtest engine.

/// The crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The version of the backtest engine IMPACT inherits and re-fills.
#[must_use]
pub fn engine_version() -> &'static str {
    wickra_backtest::version()
}

#[cfg(test)]
mod tests {
    use super::{engine_version, version};

    #[test]
    fn versions_are_reported() {
        assert!(!version().is_empty());
        assert!(!engine_version().is_empty());
    }
}
