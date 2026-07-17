//! Wickra Impact core — a market-impact backtester that walks the real historical
//! L2 order book so slippage is measured, not guessed.
//!
//! IMPACT inherits the `wickra-backtest` engine (its `StrategySpec`, run request
//! and report) and replaces only the fill stage with an order-book-walk fill
//! engine ([`book_model`]). This module wires the deterministic fill and latency
//! primitives; the spec envelope, the run loop and the `command_json` boundary
//! land in the remaining P-IMP-1 units.

pub mod book_model;
pub mod error;
pub mod latency;

pub use book_model::{fill, round_to, BookModel, Fill, Side};
pub use error::{Error, Result};

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
