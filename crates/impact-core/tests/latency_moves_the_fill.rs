//! Latency has to change which book the order fills against.
//!
//! `snapshot_index` adds `ceil(latency_ms / bar_ms)` bars on top of the
//! `next_open` shift, and returns `None` when that runs past the recorded
//! history -- in which case the order is cancelled and the report comes back
//! all zeros: no trades, no slippage, no liquidity consumed.
//!
//! That is a well-formed report, and it is what the `latency` golden case used
//! to hold: its dataset was two bars long, so a bar-0 signal targeted bar 2 and
//! never filled. The case was pinning "the order was cancelled", which is true
//! of any spec whose dataset is too short and says nothing about latency at all.
//!
//! The property below is the one the case exists for: with the same data and the
//! same strategy, a latency that spans a bar moves the fill onto a later book,
//! and the entry price says so.

use wickra_impact_core::{run, Candle, ImpactSpec, Level, OrderBook, RunData};

/// Four bars whose books step up sharply at bar 2.
fn data() -> RunData {
    let candles = [
        (100.0, 100.0, 100.0, 100.0),
        (100.0, 100.5, 99.8, 100.2),
        (101.4, 101.8, 101.2, 101.6),
        (101.6, 102.0, 101.4, 101.8),
    ];
    let books = [(99.9, 100.1), (99.9, 100.1), (101.3, 101.5), (101.5, 101.7)];

    RunData {
        candles: candles
            .iter()
            .enumerate()
            .map(|(i, &(open, high, low, close))| Candle {
                time: i64::try_from(i).expect("bar index fits an i64") * 3600,
                open,
                high,
                low,
                close,
                volume: 1000.0,
            })
            .collect(),
        capital: 100_000.0,
        books: Some(
            books
                .iter()
                .map(|&(bid, ask)| OrderBook {
                    bids: vec![Level {
                        price: bid,
                        size: 100.0,
                    }],
                    asks: vec![Level {
                        price: ask,
                        size: 100.0,
                    }],
                })
                .collect(),
        ),
        derivs: None,
        trades: None,
        reference: None,
    }
}

fn spec_with_latency(latency_ms: u64) -> ImpactSpec {
    let json = format!(
        r#"{{"strategy":{{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",
            "indicators":{{}},"entry":{{"ge":[{{"price":"close"}},0]}},
            "exit":{{"in_position":true}},
            "sizing":{{"type":"fixed_qty","qty":10.0}},
            "execution":{{"order_type":"market","fill_timing":"next_open"}}}},
            "book_model":{{"kind":"orderbook_walk"}},"participation_cap":1.0,
            "latency_ms":{latency_ms}}}"#
    );
    serde_json::from_str(&json).expect("the spec parses")
}

fn first_entry_price(latency_ms: u64) -> f64 {
    let report = run(&data(), &spec_with_latency(latency_ms)).expect("the run succeeds");
    let trade = report
        .report
        .trades
        .first()
        .unwrap_or_else(|| panic!("latency {latency_ms} produced no trade at all"));
    trade.entry_price
}

#[test]
fn a_latency_that_spans_a_bar_fills_against_the_later_book() {
    let prompt = first_entry_price(0);
    let delayed = first_entry_price(1000);

    // `next_open` puts the prompt fill on bar 1, whose top ask is 100.10.
    assert!(
        (prompt - 100.1).abs() < 1e-9,
        "a zero-latency order fills against bar 1's book, at 100.10, not {prompt}"
    );
    // `ceil(1000 / 3_600_000)` is one whole bar, so the delayed fill lands on
    // bar 2, whose top ask is 101.50. Any latency below a bar costs a full bar:
    // a fill can only land on a bar the history actually records.
    assert!(
        (delayed - 101.5).abs() < 1e-9,
        "a latency spanning a bar fills against bar 2's book, at 101.50, not {delayed}"
    );
    assert!(
        delayed > prompt,
        "the delayed fill is the worse one on a rising book"
    );
}

#[test]
fn a_latency_past_the_end_of_the_history_cancels_the_order() {
    // The other half of the same rule, and the reason the all-zero report is a
    // legitimate shape rather than a bug: an order that would land past the last
    // recorded bar has no book to fill against, so it is cancelled rather than
    // filled at a price nobody observed.
    let report = run(&data(), &spec_with_latency(10 * 3_600_000)).expect("the run succeeds");
    assert!(
        report.report.trades.is_empty(),
        "an order delayed past the recorded history cannot fill"
    );
    assert_eq!(
        report.impact_stats.avg_slippage_bps, 0.0,
        "and it consumes no liquidity on the way out"
    );
}
