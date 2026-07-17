//! The core proof of the product: filling against the real order book is not the
//! same as a naive backtest. Over the same thin-book data, the order-book walk
//! pays measurable slippage the naive engine (which fills at one reference price)
//! never sees; over a deep book the two converge.
#![allow(clippy::float_cmp)]

use std::fs;
use std::path::PathBuf;

use impact_core::{run, ImpactSpec, RunData};

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../golden")
}

fn load(name: &str) -> (ImpactSpec, RunData) {
    let dir = golden_dir();
    let spec = serde_json::from_str(
        &fs::read_to_string(dir.join("specs").join(format!("{name}.json"))).unwrap(),
    )
    .unwrap();
    let data = serde_json::from_str(
        &fs::read_to_string(dir.join("data").join(format!("{name}.json"))).unwrap(),
    )
    .unwrap();
    (spec, data)
}

#[test]
fn the_walk_pays_impact_the_naive_engine_never_sees() {
    let (spec, data) = load("thin_book");

    // The impact run walks the thin ask ladder.
    let impact = run(&data, &spec).unwrap();
    let walked_entry = impact.report.trades[0].entry_price;

    // The naive engine fills the same signal at the reference price (next_open),
    // charging zero slippage — the embedded strategy carries no slippage cost.
    let naive = wickra_backtest::run_with_capital(&spec.strategy, &data.candles, data.capital)
        .expect("naive engine run");
    let naive_entry = naive.trades[0].entry_price;

    assert!(
        walked_entry > naive_entry,
        "the walked entry ({walked_entry}) must be worse than the naive entry ({naive_entry})"
    );
    assert_eq!(walked_entry, 100.44, "the thin-book walk fills at the VWAP");
    assert!(
        impact.impact_stats.avg_slippage_bps > 0.0,
        "a thin book must show positive slippage"
    );
}

#[test]
fn a_deep_book_shows_no_market_impact() {
    let (thin_spec, thin_data) = load("thin_book");
    let (deep_spec, deep_data) = load("deep_book");

    let thin = run(&thin_data, &thin_spec).unwrap();
    let deep = run(&deep_data, &deep_spec).unwrap();

    // A deep book absorbs the whole order at the inside price: no walk up the
    // ladder, so far less slippage than the thin book.
    assert!(
        deep.impact_stats.avg_slippage_bps < thin.impact_stats.avg_slippage_bps,
        "a deep book must pay less impact than a thin one"
    );
    assert_eq!(
        deep.impact_stats.orders_partially_filled, 0,
        "a deep book fills every order in full"
    );
}
