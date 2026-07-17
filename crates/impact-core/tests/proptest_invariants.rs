//! Property tests for the fill engine: over random books, quantities and caps the
//! fill never panics, conserves quantity, produces finite non-negative outputs,
//! and is monotone in the desired quantity (buying more never fills cheaper).

use impact_core::book_model::fill;
use impact_core::{BookModel, Level, OrderBook, Side};
use proptest::prelude::*;

/// A strategy for an ascending ask ladder: `n` levels of positive price and size,
/// prices strictly increasing from a positive base.
fn ask_ladder() -> impl Strategy<Value = Vec<(f64, f64)>> {
    prop::collection::vec((0.01f64..5.0, 0.1f64..100.0), 1..8).prop_map(|steps| {
        let mut price = 100.0;
        steps
            .into_iter()
            .map(|(dp, size)| {
                price += dp;
                (price, size)
            })
            .collect()
    })
}

fn book_from(asks: &[(f64, f64)]) -> OrderBook {
    OrderBook {
        // A single deep bid keeps the mid finite; only the asks matter for a buy.
        bids: vec![Level {
            price: 99.0,
            size: 1000.0,
        }],
        asks: asks
            .iter()
            .map(|&(price, size)| Level { price, size })
            .collect(),
    }
}

proptest! {
    #[test]
    fn fill_conserves_quantity_and_stays_finite(
        asks in ask_ladder(),
        qty in 0.1f64..500.0,
        cap in 0.01f64..=1.0,
    ) {
        let book = book_from(&asks);
        let f = fill(Side::Buy, qty, 100.0, &book, &BookModel::OrderbookWalk { levels: None }, cap);

        prop_assert!(f.price.is_finite(), "price must be finite");
        prop_assert!(f.notional.is_finite() && f.notional >= 0.0, "notional finite and >= 0");
        prop_assert!(f.slippage_bps.is_finite(), "slippage must be finite");
        prop_assert!(f.qty_filled >= 0.0 && f.qty_unfilled >= 0.0, "quantities non-negative");
        // Conservation up to the 1e-8 rounding grid.
        prop_assert!(
            (f.qty_filled + f.qty_unfilled - qty).abs() < 1e-6,
            "filled + unfilled must equal the desired qty (got {} + {} vs {})",
            f.qty_filled, f.qty_unfilled, qty
        );
    }

    #[test]
    fn buying_more_never_fills_cheaper(
        asks in ask_ladder(),
        base in 0.1f64..50.0,
        extra in 0.1f64..50.0,
    ) {
        let book = book_from(&asks);
        let model = BookModel::OrderbookWalk { levels: None };
        let small = fill(Side::Buy, base, 100.0, &book, &model, 1.0);
        let large = fill(Side::Buy, base + extra, 100.0, &book, &model, 1.0);
        // A larger buy walks at least as far up the ladder: its VWAP cannot be
        // lower than the smaller buy's (monotone market impact).
        prop_assert!(
            large.price >= small.price - 1e-9,
            "buying more filled cheaper: {} for {} vs {} for {}",
            large.price, base + extra, small.price, base
        );
    }
}
