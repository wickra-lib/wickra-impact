//! `orderbook_walk` needs a book, and says so.
//!
//! This is the one way the repository's claim can fail quietly. Every other fill
//! model prices an order from a formula and needs no book; `orderbook_walk`
//! walks the real one, and that walk is the whole reason to prefer this engine
//! over a plain backtest.
//!
//! If a bookless run fell through to the empty-book fallback the analytic models
//! use, the report would show **zero slippage** — from the model whose entire
//! purpose is to find some — and nothing downstream could tell "no impact" from
//! "no book". So the refusal is the behaviour under test, not an edge case
//! beside it.

use wickra_impact_core::{run, Candle, ImpactSpec, RunData};

const BARS: usize = 60;

fn candles() -> Vec<Candle> {
    (0..BARS)
        .map(|i| {
            // Converted rather than cast: the workspace denies both the
            // precision loss of `usize as f64` and the wrap of `usize as i64`.
            let index = i64::try_from(i).expect("bar index fits an i64");
            let t = f64::from(u32::try_from(i).expect("bar index fits a u32"));
            let close = 100.0 + (t * 0.4).sin() * 5.0;
            Candle {
                time: 1_700_000_000 + index * 3600,
                open: close - 0.2,
                high: close + 0.6,
                low: close - 0.6,
                close,
                volume: 5_000.0,
            }
        })
        .collect()
}

fn spec_with(model: &str) -> ImpactSpec {
    let json = format!(
        r#"{{"strategy":{{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",
            "indicators":{{}},"entry":{{"ge":[{{"price":"close"}},0]}},
            "exit":{{"in_position":true}},
            "sizing":{{"type":"fixed_qty","qty":10.0}},
            "execution":{{"order_type":"market","fill_timing":"next_open"}}}},
            "book_model":{model},"participation_cap":1.0,"latency_ms":0}}"#
    );
    serde_json::from_str(&json).expect("parse spec")
}

fn bookless() -> RunData {
    RunData {
        candles: candles(),
        capital: 100_000.0,
        books: None,
        derivs: None,
        trades: None,
        reference: None,
    }
}

#[test]
fn the_walk_refuses_a_run_with_no_book() {
    let spec = spec_with(r#"{"kind":"orderbook_walk"}"#);
    let Err(err) = run(&bookless(), &spec) else {
        panic!("a walk with no book must be refused, not answered with zero slippage");
    };
    assert!(
        err.to_string().contains("book"),
        "the refusal names the book: {err}"
    );
}

#[test]
fn an_analytic_model_needs_no_book() {
    // The empty-book fallback exists for these: they price the order from a
    // curve, so a bookless run is the normal case rather than a hole.
    for model in [
        r#"{"kind":"linear_impact","coef":0.1}"#,
        r#"{"kind":"square_root","coef":0.5}"#,
    ] {
        let spec = spec_with(model);
        run(&bookless(), &spec)
            .unwrap_or_else(|e| panic!("{model} prices from a curve and should run: {e}"));
    }
}
