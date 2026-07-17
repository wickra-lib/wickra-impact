//! Load a request, run the impact backtest and render the result.

use crate::args::{Args, Format};
use impact_core::{run, ImpactReport, ImpactSpec, RunData};
use serde::Deserialize;
use std::io::Read;

/// A request bundle: the spec plus the run data.
#[derive(Deserialize)]
struct Request {
    spec: ImpactSpec,
    data: RunData,
}

/// Run the CLI: load the request per the arguments, run the backtest and render.
///
/// # Errors
///
/// Returns a message if the inputs cannot be read, parsed or run.
pub fn run_cli(args: &Args) -> Result<String, String> {
    let (spec, data) = load(args)?;
    let report = run(&data, &spec).map_err(|e| e.to_string())?;
    Ok(match args.format {
        Format::Json => serde_json::to_string(&report).map_err(|e| e.to_string())?,
        Format::Text => render_text(&report),
    })
}

fn load(args: &Args) -> Result<(ImpactSpec, RunData), String> {
    if let Some(path) = &args.request {
        let req: Request = read_json(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)?;
        return Ok((req.spec, req.data));
    }
    if args.stdin {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| e.to_string())?;
        let req: Request = read_json(&buf)?;
        return Ok((req.spec, req.data));
    }
    match (&args.spec, &args.data) {
        (Some(spec_path), Some(data_path)) => {
            let spec = read_json(&std::fs::read_to_string(spec_path).map_err(|e| e.to_string())?)?;
            let data = read_json(&std::fs::read_to_string(data_path).map_err(|e| e.to_string())?)?;
            Ok((spec, data))
        }
        _ => Err("provide --request, --spec + --data, or --stdin".into()),
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, String> {
    serde_json::from_str(s).map_err(|e| e.to_string())
}

fn render_text(report: &ImpactReport) -> String {
    let m = &report.report.metrics;
    let s = &report.impact_stats;
    format!(
        "backtest summary\n\
         ================\n\
         pnl            {:>14.2}\n\
         return         {:>13.2}%\n\
         trades         {:>14}\n\
         win rate       {:>13.2}%\n\
         sharpe         {:>14.4}\n\
         max drawdown   {:>13.2}%\n\
         fees paid      {:>14.2}\n\
         \n\
         market impact\n\
         =============\n\
         avg slippage       {:>10.4} bps\n\
         liquidity consumed {:>13.2}\n\
         partial fills      {:>10}",
        m.pnl,
        m.return_pct,
        m.num_trades,
        m.win_rate,
        m.sharpe,
        m.max_drawdown,
        report.report.fees_paid,
        s.avg_slippage_bps,
        s.liquidity_consumed,
        s.orders_partially_filled,
    )
}
