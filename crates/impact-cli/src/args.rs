//! Command-line arguments for `wickra-impact`.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Backtest a strategy against the real historical L2 order book, so slippage is
/// measured, not guessed.
#[derive(Parser, Debug)]
#[command(name = "wickra-impact", version, about)]
pub struct Args {
    /// Path to a request bundle (`{"spec": <ImpactSpec>, "data": <RunData>}`).
    #[arg(long, value_name = "PATH", conflicts_with_all = ["spec", "data", "stdin"])]
    pub request: Option<PathBuf>,

    /// Path to the impact spec (`ImpactSpec` JSON). Pair with `--data`.
    #[arg(long, value_name = "PATH", requires = "data")]
    pub spec: Option<PathBuf>,

    /// Path to the run data (`RunData` JSON). Pair with `--spec`.
    #[arg(long, value_name = "PATH", requires = "spec")]
    pub data: Option<PathBuf>,

    /// Read a request bundle as JSON from stdin.
    #[arg(long)]
    pub stdin: bool,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    pub format: Format,
}

/// The output format.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Format {
    /// A human-readable summary with the market-impact block.
    Text,
    /// The raw `ImpactReport` JSON.
    Json,
}
