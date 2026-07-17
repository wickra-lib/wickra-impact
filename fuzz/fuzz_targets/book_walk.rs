#![no_main]
//! Fuzz the fill engine: a JSON-encoded order (quantity, reference, cap) and a
//! book (bid/ask ladders) drive `book_model::fill`. No input may panic and every
//! output must be finite with conserved, non-negative quantities.

use impact_core::book_model::fill;
use impact_core::{BookModel, Level, OrderBook, Side};
use libfuzzer_sys::fuzz_target;
use serde::Deserialize;

#[derive(Deserialize)]
struct Order {
    qty: f64,
    reference: f64,
    cap: f64,
    asks: Vec<(f64, f64)>,
    #[serde(default)]
    bids: Vec<(f64, f64)>,
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(order) = serde_json::from_str::<Order>(text) else {
        return;
    };
    // Bound the inputs so the fuzzer cannot force pathological magnitudes.
    if !order.qty.is_finite() || !order.reference.is_finite() || !order.cap.is_finite() {
        return;
    }
    let qty = order.qty.clamp(0.0, 1e6);
    let reference = order.reference.clamp(0.0, 1e9);
    let cap = if order.cap.is_finite() && order.cap > 0.0 && order.cap <= 1.0 {
        order.cap
    } else {
        1.0
    };
    let to_levels = |v: &[(f64, f64)]| {
        v.iter()
            .filter(|(p, s)| p.is_finite() && s.is_finite())
            .map(|&(price, size)| Level {
                price: price.clamp(0.0, 1e9),
                size: size.clamp(0.0, 1e6),
            })
            .collect::<Vec<_>>()
    };
    let book = OrderBook {
        bids: to_levels(&order.bids),
        asks: to_levels(&order.asks),
    };

    let f = fill(
        Side::Buy,
        qty,
        reference,
        &book,
        &BookModel::OrderbookWalk { levels: None },
        cap,
    );
    assert!(f.price.is_finite(), "fill price must be finite");
    assert!(f.notional.is_finite() && f.notional >= 0.0, "notional finite and >= 0");
    assert!(f.qty_filled >= 0.0 && f.qty_unfilled >= 0.0, "quantities non-negative");
});
