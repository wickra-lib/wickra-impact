//! The crate error type.

/// Everything that can go wrong building or running an impact backtest.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A JSON payload could not be parsed.
    #[error("parse: {0}")]
    Parse(String),
    /// The `ImpactSpec` (or its embedded strategy) is invalid.
    #[error("bad spec: {0}")]
    BadSpec(String),
    /// The run data is inconsistent (missing books, mismatched feed lengths).
    #[error("data: {0}")]
    Data(String),
    /// The inherited backtest engine rejected the run.
    #[error("backtest: {0}")]
    Backtest(String),
}

/// The crate result alias.
pub type Result<T> = core::result::Result<T, Error>;

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Parse(e.to_string())
    }
}

impl From<wickra_backtest::BacktestError> for Error {
    fn from(e: wickra_backtest::BacktestError) -> Self {
        Error::Backtest(e.to_string())
    }
}
