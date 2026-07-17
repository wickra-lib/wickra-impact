# Architecture

Wickra Impact is a market-impact backtester. It **inherits the `wickra-backtest`
engine** and replaces **only the fill stage** with an engine that walks the real
historical L2 order book.

## Workspace

```
crates/impact-core   the library: inherits the backtest types, replaces the fill
                     stage with an order-book walk, exposes command_json
crates/impact-cli    the wickra-impact reference CLI
crates/impact-bench  criterion micro-benchmarks (fills per second)
bindings/*           the language surfaces (c, python, node, wasm; C ABI hub
                     extends to c++, c#, go, java, r)
```

## Inheritance, not reimplementation

From `wickra-backtest` Impact reuses **verbatim** (as a git dependency, version
pinned): `StrategySpec`, `RunRequest`, `BacktestReport`, `Metrics`, `Trade`,
`Portfolio`, and the `OrderBook` / `Level` types. The streaming engine drives the
run exactly as a plain backtest does — bar by bar, feeding candles and the
recorded books.

## The fill stage IMPACT replaces

A plain backtest derives a fill price from a single number (close, or a fixed
slippage estimate). Impact instead **walks the order across the recorded book**:
starting at the best level, it consumes available size at each price until the
order quantity is filled, and reports the size-weighted average price. The gap
between that price and the naive fill is the market impact the strategy actually
paid. The walk is deterministic: levels are held best-first in a stable order,
reductions run serially in level order, and the resulting price is rounded onto a
fixed grid before it enters the report — so the same request yields a
byte-identical report on every run and in every binding.

## The command boundary

Every binding calls one entry point, `command_json`: a JSON command string in,
a JSON report string out, forwarded verbatim. There is no per-language logic, so
all ten languages return identical bytes.
