//! Wickra Impact core — a market-impact backtester that walks the real historical
//! L2 order book so slippage is measured, not guessed.
//!
//! IMPACT inherits the `wickra-backtest` engine (its `StrategySpec`, run request
//! and report) and replaces only the fill stage with an order-book-walk fill
//! engine ([`book_model`]). Point it at a recorded universe, `run` a strategy, and
//! it reconstructs the same `BacktestReport` — but with the real fill prices and a
//! market-impact statistics block.
//!
//! The single data-driven entry point is [`Impact::command_json`], forwarded
//! verbatim by every language binding for a byte-identical result.

pub mod book_model;
pub mod config;
pub mod error;
pub mod impact;
pub mod latency;
pub mod run;
pub mod spec;

pub use book_model::{fill, round_to, BookModel, Fill, Side};
pub use config::Config;
pub use error::{Error, Result};
pub use impact::Impact;
pub use run::{run, ImpactReport, ImpactStats, RunData};
pub use spec::ImpactSpec;

// Re-export the inherited backtest types so consumers pull everything from one
// crate.
pub use wickra_backtest::core::data::Level;
pub use wickra_backtest::core::metrics::Metrics;
pub use wickra_backtest::core::portfolio::Trade;
pub use wickra_backtest::{BacktestReport, Candle, EquityPoint, OrderBook, StrategySpec};

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
