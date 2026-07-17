# The order-book walk: why every other backtest lies

A plain backtest fills your order at one price — the close, the next open, or that
price plus a fixed slippage estimate — as if your size were invisible. But a real
market order does not transact at a single price: it lifts the book, taking the
cheapest offers first and paying up the ladder until it is filled. The gap between
those two is **market impact**, and for any non-trivial size it is the difference
between a backtest that looks profitable and one that is.

## The `thin_book` worked example

The golden `thin_book` case makes the lie visible. A buy-and-hold strategy signals
on bar 0 and fills at bar 1's open against this ask ladder:

| Level | Price   | Size |
|-------|---------|------|
| 1     | 100.1   | 3    |
| 2     | 100.3   | 3    |
| 3     | 100.8   | 4    |

The order is for **10 units**. A naive backtest fills all 10 at the `100.1` inside
price. The walk instead consumes:

```
3 @ 100.1  = 300.3
3 @ 100.3  = 300.9
4 @ 100.8  = 403.2
           ─────────
10 units   = 1004.4   →  VWAP = 100.44
```

The fill is **100.44**, not 100.1 — a slippage of `(100.44 − 100.1) / 100.1`, or
**44 basis points**, that the naive backtest never charged. `ImpactStats` reports
`avg_slippage_bps = 44.0`, `liquidity_consumed = 1004.4`, and (because the book
held the full order) `orders_partially_filled = 0`.

## Deep books converge

Run the same order against `deep_book` — 1000 units at the inside — and the walk
never leaves the first level: the fill is the inside price, the impact is zero, and
the result matches the naive backtest. That is the point: Impact does not penalise
you when liquidity is deep; it charges you exactly when, and as much as, the real
book would have.

Reproduce:

```bash
cargo run -p impact-cli -- --spec golden/specs/thin_book.json --data golden/data/thin_book.json --format text
```
