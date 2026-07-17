//! `wickra-impact` — backtest a strategy against the real historical L2 order
//! book, measuring the market impact every other backtest ignores.

mod args;
mod run;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = args::Args::parse();
    match run::run_cli(&args) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
