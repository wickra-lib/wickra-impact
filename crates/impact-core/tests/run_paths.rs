//! The paths through `run` the golden corpus never walks.
//!
//! The six golden cases all describe one shape: a long, market-ordered,
//! next-open entry over a book feed, exited by the spec's own exit condition.
//! That leaves the refusals and half the execution model unmeasured -- the
//! close-timing fill, the short side, the intrabar stop / take-profit /
//! trailing exits, liquidation, funding off the derivatives feed, and every
//! way a caller can hand `run` a dataset that does not line up.
//!
//! Each test below drives one of those and asserts what the report says about
//! it, not merely that the call returned.

use wickra_backtest::core::data::DerivativesTick;
use wickra_impact_core::{run, Candle, ImpactSpec, Level, OrderBook, RunData};

/// A rising path: every bar closes above the last, so a long entry is in
/// profit and a short is under water.
fn rising(n: usize) -> Vec<Candle> {
    (0..n)
        .map(|i| {
            let base = 100.0 + f64::from(u32::try_from(i).expect("bar index fits a u32"));
            Candle {
                time: i64::try_from(i).expect("bar index fits an i64") * 3600,
                open: base,
                high: base + 0.6,
                low: base - 0.4,
                close: base + 0.5,
                volume: 1000.0,
            }
        })
        .collect()
}

fn books(n: usize, candles: &[Candle]) -> Vec<OrderBook> {
    candles
        .iter()
        .take(n)
        .map(|c| OrderBook {
            bids: vec![Level {
                price: c.close - 0.1,
                size: 500.0,
            }],
            asks: vec![Level {
                price: c.close + 0.1,
                size: 500.0,
            }],
        })
        .collect()
}

fn data_with(candles: Vec<Candle>) -> RunData {
    let bks = books(candles.len(), &candles);
    RunData {
        candles,
        capital: 100_000.0,
        books: Some(bks),
        derivs: None,
        trades: None,
        reference: None,
    }
}

/// The base strategy every case starts from, with the overrides merged in, so
/// each test differs from the others only in what it is about.
fn spec(overrides: &serde_json::Value, book_model: &str) -> ImpactSpec {
    let mut strategy = serde_json::json!({
        "spec_version": 1,
        "symbol": "IMPACT",
        "timeframe": "1h",
        "indicators": {},
        "entry": { "ge": [{ "price": "close" }, 0] },
        "exit": { "in_position": false },
        "sizing": { "type": "fixed_qty", "qty": 10.0 },
        "execution": { "order_type": "market", "fill_timing": "next_open" }
    });
    for (key, value) in overrides.as_object().expect("overrides are an object") {
        strategy[key] = value.clone();
    }
    let spec = serde_json::json!({
        "strategy": strategy,
        "book_model": serde_json::from_str::<serde_json::Value>(book_model).unwrap(),
        "participation_cap": 1.0,
        "latency_ms": 0
    });
    serde_json::from_value(spec.clone()).unwrap_or_else(|e| {
        panic!(
            "the spec parses: {e}
{spec}"
        )
    })
}

const WALK: &str = r#"{"kind":"orderbook_walk"}"#;
const LINEAR: &str = r#"{"kind":"linear_impact","coef":0.1}"#;

// ---------------------------------------------------------------- refusals

#[test]
fn an_empty_dataset_is_refused() {
    let mut data = data_with(rising(4));
    data.candles.clear();
    data.books = Some(Vec::new());
    let err = run(&data, &spec(&serde_json::json!({}), LINEAR)).expect_err("no candles is refused");
    assert!(err.to_string().contains("no candles"), "{err}");
}

#[test]
fn a_book_feed_that_does_not_line_up_is_refused() {
    let mut data = data_with(rising(4));
    data.books = Some(books(2, &rising(4)));
    let err = run(&data, &spec(&serde_json::json!({}), LINEAR))
        .expect_err("a short book feed is refused");
    assert!(err.to_string().contains("books length"), "{err}");
}

#[test]
fn the_walk_model_requires_a_book_feed() {
    let mut data = data_with(rising(4));
    data.books = None;
    let err = run(&data, &spec(&serde_json::json!({}), WALK))
        .expect_err("orderbook_walk without books is refused");
    assert!(err.to_string().contains("requires a book feed"), "{err}");
    // The other models do not: they price the impact from the candle alone.
    assert!(run(&data, &spec(&serde_json::json!({}), LINEAR)).is_ok());
}

/// One mutation of a dataset: the feed a case makes too short.
type Mutate = Box<dyn Fn(&mut RunData)>;

#[test]
fn every_optional_feed_must_match_the_candles() {
    let candles = rising(4);
    let short_trades: Vec<Vec<wickra_backtest::core::data::TradePrint>> = vec![Vec::new(); 3];
    let cases: Vec<(&str, Mutate)> = vec![
        (
            "trades",
            Box::new(move |d: &mut RunData| d.trades = Some(short_trades.clone())),
        ),
        (
            "derivs",
            Box::new(|d: &mut RunData| d.derivs = Some(vec![deriv(0.0); 3])),
        ),
        (
            "reference",
            Box::new(|d: &mut RunData| d.reference = Some(rising(3))),
        ),
    ];
    for (name, mutate) in cases {
        let mut data = data_with(candles.clone());
        mutate(&mut data);
        let err = run(&data, &spec(&serde_json::json!({}), LINEAR))
            .err()
            .unwrap_or_else(|| panic!("a {name} feed of the wrong length must be refused"));
        assert!(err.to_string().contains(name), "{name}: {err}");
    }
}

fn deriv(funding_rate: f64) -> DerivativesTick {
    DerivativesTick {
        funding_rate,
        mark_price: 100.0,
        index_price: 100.0,
        futures_price: 100.0,
        open_interest: 0.0,
        long_size: 0.0,
        short_size: 0.0,
        taker_buy_volume: 0.0,
        taker_sell_volume: 0.0,
        long_liquidation: 0.0,
        short_liquidation: 0.0,
        timestamp: 0,
    }
}

// ---------------------------------------------------------------- execution

#[test]
fn a_close_timed_fill_enters_on_the_signal_bar() {
    let data = data_with(rising(6));
    let at_close = run(
        &data,
        &spec(
            &serde_json::json!({ "execution": { "order_type": "market", "fill_timing": "close" } }),
            LINEAR,
        ),
    )
    .expect("the run succeeds");
    let at_open = run(&data, &spec(&serde_json::json!({}), LINEAR)).expect("the run succeeds");

    let close_entry = at_close.report.trades.first().expect("a trade at close");
    let open_entry = at_open.report.trades.first().expect("a trade at next open");
    // The signal fires on bar 0 either way; close timing fills at that bar's
    // close, next-open timing at bar 1's open, and the prices differ.
    assert!(
        (close_entry.entry_price - open_entry.entry_price).abs() > f64::EPSILON,
        "close {} vs next_open {}",
        close_entry.entry_price,
        open_entry.entry_price
    );
}

#[test]
fn a_short_entry_takes_the_other_side() {
    // Entry never fires; the short condition does.
    let data = data_with(rising(6));
    let short = run(
        &data,
        &spec(
            &serde_json::json!({
                "entry": { "ge": [{ "price": "close" }, 1e12] },
                "short_entry": { "ge": [{ "price": "close" }, 0] }
            }),
            LINEAR,
        ),
    )
    .expect("the run succeeds");
    let trade = short.report.trades.first().expect("a short trade");
    assert!(
        trade.qty < 0.0,
        "a short position is negative: {}",
        trade.qty
    );
}

#[test]
fn a_stop_loss_exits_inside_the_bar() {
    // A long into a falling path, stopped out at 1%.
    let mut candles = rising(6);
    for (i, c) in candles.iter_mut().enumerate().skip(1) {
        let base = 100.0 - f64::from(u32::try_from(i).expect("bar index fits a u32")) * 2.0;
        c.open = base;
        c.high = base + 0.2;
        c.low = base - 2.0;
        c.close = base - 1.5;
    }
    let data = data_with(candles);
    let report = run(
        &data,
        &spec(
            &serde_json::json!({ "risk": { "stop_loss_pct": 1.0 } }),
            LINEAR,
        ),
    )
    .expect("the run succeeds");
    let trade = report.report.trades.first().expect("a trade");
    assert_eq!(trade.reason, "stop_loss", "{trade:?}");
}

#[test]
fn a_take_profit_exits_inside_the_bar() {
    let data = data_with(rising(8));
    let report = run(
        &data,
        &spec(
            &serde_json::json!({ "risk": { "take_profit_pct": 0.5 } }),
            LINEAR,
        ),
    )
    .expect("the run succeeds");
    let trade = report.report.trades.first().expect("a trade");
    assert_eq!(trade.reason, "take_profit", "{trade:?}");
}

#[test]
fn funding_is_charged_from_the_derivatives_feed() {
    let candles = rising(6);
    let mut with_funding = data_with(candles.clone());
    with_funding.derivs = Some(vec![deriv(0.01); candles.len()]);
    let mut without = data_with(candles);
    without.derivs = Some(vec![deriv(0.0); 6]);

    let s = spec(
        &serde_json::json!({ "costs": { "taker_bps": 0.0, "maker_bps": 0.0, "funding": true } }),
        LINEAR,
    );
    let charged = run(&with_funding, &s).expect("the run succeeds");
    let free = run(&without, &s).expect("the run succeeds");
    assert!(
        charged.report.equity.last().unwrap().equity < free.report.equity.last().unwrap().equity,
        "a funding rate has to cost the position something"
    );
}
