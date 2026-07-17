//! The fill engine — the market-impact moat.
//!
//! Where a plain backtest fills an order at one reference price, IMPACT walks the
//! order across the real historical L2 order book (or an analytic impact curve)
//! and reports the volume-weighted fill price, the unfilled remainder and the
//! slippage the order actually paid. Everything here is **deterministic**:
//! reductions run serially in a fixed level order and every output is rounded onto
//! a fixed grid, so the same inputs yield byte-identical results on every run and
//! in every language binding.

use serde::{Deserialize, Serialize};
use wickra_backtest::OrderBook;

/// Round `x` onto the fixed `1e-8` grid used across every serialised output.
#[must_use]
pub fn round_to(x: f64, step: f64) -> f64 {
    if !x.is_finite() {
        return 0.0;
    }
    (x / step).round() * step
}

/// Which side of the book an order lifts: a buy walks the asks, a sell the bids.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// A buy: lifts the ask side, pays up.
    Buy,
    /// A sell: hits the bid side, pays down.
    Sell,
}

impl Side {
    /// `+1.0` for a buy (fills above mid), `-1.0` for a sell (fills below mid).
    #[must_use]
    pub fn sign(self) -> f64 {
        match self {
            Side::Buy => 1.0,
            Side::Sell => -1.0,
        }
    }
}

/// The fill / market-impact model — data, not code, so it crosses the C ABI.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BookModel {
    /// Walk the real historical L2 book, level by level (the moat).
    OrderbookWalk {
        /// How many levels to walk; `None` walks all provided levels.
        #[serde(default)]
        levels: Option<usize>,
    },
    /// Analytic linear impact: `fill = ref * (1 +/- coef * (qty / depth))`.
    LinearImpact {
        /// Impact coefficient (finite, positive).
        coef: f64,
    },
    /// Analytic square-root impact: `fill = ref * (1 +/- coef * sqrt(qty / depth))`.
    SquareRoot {
        /// Impact coefficient (finite, positive).
        coef: f64,
    },
}

/// The outcome of filling one order: the volume-weighted price, how much filled
/// and how much did not, the notional consumed and the slippage vs. mid in bps.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Fill {
    /// Volume-weighted average fill price (the reference price if nothing filled).
    pub price: f64,
    /// Quantity actually filled.
    pub qty_filled: f64,
    /// Quantity left unfilled (book too thin or the participation cap bound).
    pub qty_unfilled: f64,
    /// Notional liquidity consumed (`Σ take * price`).
    pub notional: f64,
    /// Slippage of the fill price vs. the book mid, signed, in basis points.
    pub slippage_bps: f64,
}

/// The mid price of a book, or `None` if either side is empty or the mid is not
/// strictly positive.
fn mid(book: &OrderBook) -> Option<f64> {
    match (book.bids.first(), book.asks.first()) {
        (Some(bid), Some(ask)) => {
            let m = f64::midpoint(bid.price, ask.price);
            (m > 0.0).then_some(m)
        }
        _ => None,
    }
}

/// Slippage of `vwap` vs. `reference`, signed by side, in basis points; `0.0` if
/// the reference is not usable or nothing filled.
fn slippage_bps(side: Side, vwap: f64, reference: Option<f64>, filled: f64) -> f64 {
    match reference {
        Some(r) if r > 0.0 && filled > 0.0 => round_to(1e4 * side.sign() * (vwap - r) / r, 1e-8),
        _ => 0.0,
    }
}

/// Fill `qty` (> 0) of `side` at `reference` against `book` under `model`, taking
/// at most `cap` (in `(0, 1]`) of the available liquidity. Serial and
/// deterministic; a thin or empty book falls back to `price = reference` with
/// zero slippage.
#[must_use]
pub fn fill(
    side: Side,
    qty: f64,
    reference: f64,
    book: &OrderBook,
    model: &BookModel,
    cap: f64,
) -> Fill {
    let levels = match side {
        Side::Buy => &book.asks,
        Side::Sell => &book.bids,
    };
    let mid = mid(book);

    match *model {
        BookModel::OrderbookWalk { levels: depth_cap } => {
            let take_levels = depth_cap.unwrap_or(levels.len()).min(levels.len());
            let walked = &levels[..take_levels];
            let avail: f64 = walked.iter().map(|l| l.size).sum::<f64>() * cap;
            let fillable = qty.min(avail);
            let mut remaining = fillable;
            let mut cost = 0.0;
            let mut filled = 0.0;
            for level in walked {
                if remaining <= 0.0 {
                    break;
                }
                let take = remaining.min(level.size);
                cost += take * level.price;
                filled += take;
                remaining -= take;
            }
            let vwap = if filled > 0.0 {
                cost / filled
            } else {
                reference
            };
            Fill {
                price: round_to(vwap, 1e-8),
                qty_filled: round_to(filled, 1e-8),
                qty_unfilled: round_to(qty - filled, 1e-8),
                notional: round_to(cost, 1e-8),
                slippage_bps: slippage_bps(side, vwap, mid.or(Some(reference)), filled),
            }
        }
        BookModel::LinearImpact { coef } | BookModel::SquareRoot { coef } => {
            let depth: f64 = levels.iter().map(|l| l.size).sum();
            let fillable = qty.min(cap * depth);
            let (vwap, filled) = if depth > 0.0 && fillable > 0.0 {
                let ratio = fillable / depth;
                let shift = match *model {
                    BookModel::SquareRoot { .. } => coef * ratio.sqrt(),
                    _ => coef * ratio,
                };
                (reference * (1.0 + side.sign() * shift), fillable)
            } else {
                (reference, 0.0)
            };
            Fill {
                price: round_to(vwap, 1e-8),
                qty_filled: round_to(filled, 1e-8),
                qty_unfilled: round_to(qty - filled, 1e-8),
                notional: round_to(filled * vwap, 1e-8),
                slippage_bps: slippage_bps(side, vwap, mid.or(Some(reference)), filled),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::{fill, round_to, BookModel, Side};
    use wickra_backtest::core::{Level, OrderBook};

    fn book(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> OrderBook {
        let lv = |s: &[(f64, f64)]| {
            s.iter()
                .map(|&(price, size)| Level { price, size })
                .collect()
        };
        OrderBook {
            bids: lv(bids),
            asks: lv(asks),
        }
    }

    #[test]
    fn worked_example_thin_book_is_44_bps() {
        // Buy 10 against asks 3@100.1 + 3@100.3 + 4@100.8; mid = 100.0.
        let b = book(
            &[(99.9, 100.0)],
            &[(100.1, 3.0), (100.3, 3.0), (100.8, 4.0)],
        );
        let f = fill(
            Side::Buy,
            10.0,
            100.0,
            &b,
            &BookModel::OrderbookWalk { levels: None },
            1.0,
        );
        assert_eq!(f.price, 100.44);
        assert_eq!(f.qty_filled, 10.0);
        assert_eq!(f.qty_unfilled, 0.0);
        assert_eq!(f.notional, 1004.4);
        assert_eq!(f.slippage_bps, 44.0);
    }

    #[test]
    fn participation_cap_leaves_a_partial() {
        // cap 0.5 of 10 available -> fill 5, 5 unfilled.
        let b = book(
            &[(99.9, 100.0)],
            &[(100.1, 3.0), (100.3, 3.0), (100.8, 4.0)],
        );
        let f = fill(
            Side::Buy,
            10.0,
            100.0,
            &b,
            &BookModel::OrderbookWalk { levels: None },
            0.5,
        );
        assert_eq!(f.qty_filled, 5.0);
        assert_eq!(f.qty_unfilled, 5.0);
    }

    #[test]
    fn empty_side_falls_back_to_reference() {
        let b = book(&[(99.9, 100.0)], &[]);
        let f = fill(
            Side::Buy,
            10.0,
            100.0,
            &b,
            &BookModel::OrderbookWalk { levels: None },
            1.0,
        );
        assert_eq!(f.price, 100.0);
        assert_eq!(f.qty_filled, 0.0);
        assert_eq!(f.slippage_bps, 0.0);
    }

    #[test]
    fn linear_impact_moves_the_price_up_for_a_buy() {
        let b = book(&[(99.9, 100.0)], &[(100.1, 100.0)]);
        let f = fill(
            Side::Buy,
            10.0,
            100.0,
            &b,
            &BookModel::LinearImpact { coef: 0.1 },
            1.0,
        );
        // ratio = 10/100 = 0.1, shift = 0.1*0.1 = 0.01 -> 101.0
        assert_eq!(f.price, 101.0);
        assert_eq!(f.qty_filled, 10.0);
    }

    #[test]
    fn square_root_impact_matches_formula() {
        let b = book(&[(99.9, 100.0)], &[(100.1, 100.0)]);
        let f = fill(
            Side::Sell,
            10.0,
            100.0,
            &b,
            &BookModel::SquareRoot { coef: 0.5 },
            1.0,
        );
        // sell: shift = 0.5*sqrt(0.1) -> price below reference
        let expected = round_to(100.0 * (1.0 - 0.5 * 0.1_f64.sqrt()), 1e-8);
        assert_eq!(f.price, expected);
    }

    #[test]
    fn levels_cap_bounds_the_walk() {
        // Only walk the first level -> at most 3 filled.
        let b = book(
            &[(99.9, 100.0)],
            &[(100.1, 3.0), (100.3, 3.0), (100.8, 4.0)],
        );
        let f = fill(
            Side::Buy,
            10.0,
            100.0,
            &b,
            &BookModel::OrderbookWalk { levels: Some(1) },
            1.0,
        );
        assert_eq!(f.qty_filled, 3.0);
        assert_eq!(f.qty_unfilled, 7.0);
    }

    #[test]
    fn round_to_kills_non_finite() {
        assert_eq!(round_to(f64::NAN, 1e-8), 0.0);
        assert_eq!(round_to(f64::INFINITY, 1e-8), 0.0);
        assert_eq!(round_to(f64::NEG_INFINITY, 1e-8), 0.0);
        // The 9th decimal is dropped onto the 1e-8 grid.
        assert!((round_to(1.234_567_894, 1e-8) - 1.234_567_89).abs() < 1e-12);
    }
}
