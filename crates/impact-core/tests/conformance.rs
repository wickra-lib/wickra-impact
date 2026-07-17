//! Serde and validation conformance: every public enum variant and struct
//! round-trips through JSON, the tag names are pinned, an unknown `book_model.kind`
//! is rejected, a missing book for `orderbook_walk` is a run error, and an embedded
//! `costs.slippage` is a spec error (IMPACT owns the fill stage).

use impact_core::book_model::Fill;
use impact_core::{run, BookModel, ImpactSpec, ImpactStats, RunData, Side};

fn json_round_trip<T>(value: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let text = serde_json::to_string(value).expect("serialize");
    let back: T = serde_json::from_str(&text).expect("deserialize");
    assert_eq!(&back, value, "JSON round-trip must be stable");
}

const SPEC: &str = r#"{
    "strategy": {
        "spec_version": 1, "symbol": "IMPACT", "timeframe": "1h",
        "indicators": {},
        "entry": {"ge": [{"price": "close"}, 0]},
        "exit": {"in_position": true},
        "sizing": {"type": "fixed_qty", "qty": 10.0},
        "execution": {"order_type": "market", "fill_timing": "next_open"}
    },
    "book_model": {"kind": "orderbook_walk"},
    "participation_cap": 1.0, "latency_ms": 0
}"#;

const DATA: &str = r#"{
    "candles": [
        {"time": 0, "open": 100, "high": 100, "low": 100, "close": 100, "volume": 1000},
        {"time": 3600, "open": 100, "high": 103, "low": 100, "close": 102, "volume": 1000}
    ],
    "books": [
        {"bids": [{"price": 99.9, "size": 100}], "asks": [{"price": 100.1, "size": 100}]},
        {"bids": [{"price": 99.9, "size": 100}], "asks": [{"price": 100.1, "size": 3}, {"price": 100.3, "size": 3}, {"price": 100.8, "size": 4}]}
    ]
}"#;

#[test]
fn side_variants_round_trip() {
    for s in [Side::Buy, Side::Sell] {
        json_round_trip(&s);
    }
    assert_eq!(serde_json::to_string(&Side::Buy).unwrap(), "\"buy\"");
    assert_eq!(serde_json::to_string(&Side::Sell).unwrap(), "\"sell\"");
}

#[test]
fn book_model_variants_round_trip() {
    json_round_trip(&BookModel::OrderbookWalk { levels: None });
    json_round_trip(&BookModel::OrderbookWalk { levels: Some(3) });
    json_round_trip(&BookModel::LinearImpact { coef: 0.1 });
    json_round_trip(&BookModel::SquareRoot { coef: 0.5 });
    // The tag is snake_case on `kind`.
    assert_eq!(
        serde_json::to_string(&BookModel::LinearImpact { coef: 0.1 }).unwrap(),
        "{\"kind\":\"linear_impact\",\"coef\":0.1}"
    );
}

#[test]
fn structs_round_trip() {
    json_round_trip(&ImpactStats {
        avg_slippage_bps: 44.0,
        liquidity_consumed: 1004.4,
        orders_partially_filled: 0,
    });
    json_round_trip(&Fill {
        price: 100.44,
        qty_filled: 10.0,
        qty_unfilled: 0.0,
        notional: 1004.4,
        slippage_bps: 44.0,
    });
    let spec: ImpactSpec = serde_json::from_str(SPEC).unwrap();
    json_round_trip(&spec);
}

#[test]
fn unknown_book_model_kind_is_rejected() {
    let bad = SPEC.replace("\"orderbook_walk\"", "\"teleport\"");
    assert!(
        ImpactSpec::from_json(&bad).is_err(),
        "an unknown book_model.kind must fail to parse"
    );
}

#[test]
fn missing_book_for_orderbook_walk_is_a_run_error() {
    let spec: ImpactSpec = serde_json::from_str(SPEC).unwrap();
    let mut data: RunData = serde_json::from_str(DATA).unwrap();
    data.books = None;
    assert!(
        run(&data, &spec).is_err(),
        "orderbook_walk without books must be a run error"
    );
}

#[test]
fn embedded_slippage_is_a_spec_error() {
    // A slippage set on the embedded strategy would double-count IMPACT's own
    // fill; the validator must reject it.
    let bad = SPEC.replace(
        "\"execution\": {\"order_type\": \"market\", \"fill_timing\": \"next_open\"}",
        "\"execution\": {\"order_type\": \"market\", \"fill_timing\": \"next_open\"}, \"costs\": {\"slippage\": {\"type\": \"fixed_bps\", \"bps\": 5.0}}",
    );
    let spec: ImpactSpec = serde_json::from_str(&bad).unwrap();
    assert!(
        spec.validate().is_err(),
        "an embedded costs.slippage must be a spec error"
    );
}
