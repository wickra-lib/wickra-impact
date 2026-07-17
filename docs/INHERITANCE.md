# Inheritance: what IMPACT keeps and what it overrides

Wickra Impact is not a new backtester. It is `wickra-backtest` with one stage
replaced. Knowing exactly what is inherited unchanged and what is overridden is
the key to trusting its numbers.

## Inherited unchanged

- **`StrategySpec`** — the strategy is a `wickra-backtest` spec, embedded verbatim
  in `ImpactSpec.strategy`. Indicators, entry/exit rules, sizing and execution
  settings all mean exactly what they mean in the engine.
- **Signals** — entry and exit conditions are evaluated by the engine's own
  `rules::eval_condition` over indicators built by its `registry`. No rule logic is
  reimplemented.
- **Portfolio & metrics** — position, cash, fees and the equity curve are tracked
  by the engine's `Portfolio`; the headline metrics (PnL, return, Sharpe, max
  drawdown, win rate) come from its `metrics::compute`.
- **`BacktestReport`** — the result carries the inherited report shape; the impact
  run wraps it (`ImpactReport { report, impact_stats }`) rather than replacing it.

## Overridden

- **The fill stage.** Where the engine applies a fixed slippage model to a single
  reference price, IMPACT walks the order across the real L2 book (or an analytic
  impact curve). This is the whole product.
- **Required inputs.** A `RunData` for the `orderbook_walk` model must carry
  per-bar `books`; a plain backtest needs only candles.
- **New outputs.** `ImpactStats` adds `avg_slippage_bps`, `liquidity_consumed` and
  `orders_partially_filled` — statistics that only exist because fills now have
  structure.

## Documented deviations

The rebuilt bar loop reproduces the engine faithfully, with three deliberate
modelling choices where a walk differs from a fixed slippage: an entry is sized at
the reference price (the walk price depends on the sized quantity, so sizing cannot
use the post-fill price); a book too thin for the whole order leaves the remainder
unfilled (a partial entry); and an exit always closes the full position at the
walked price. **All three collapse to the engine's behaviour when the book is deep
and the impact is zero** — the invariant the fidelity test pins.

## Why this matters

Because the strategy, the signals and the metrics are the engine's own, a
side-by-side of an IMPACT run and a plain `wickra-backtest` run isolates exactly
one variable: the cost of filling against a real book. That difference is the
market impact your strategy would have paid, and nothing else.
