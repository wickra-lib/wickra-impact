# Benchmarks

The headline figure is **bars per second** — the rate at which the engine runs
one strategy signal and resolves the resulting market order against a recorded L2
order book, level by level, into a size-weighted fill price. This is the work
IMPACT adds on top of the inherited `wickra-backtest` engine.

Reproduce with:

```bash
cargo bench -p impact-bench            # parallel-capable engine
cargo bench -p impact-bench --no-default-features   # single-threaded (WASM) path
```

## Measured (reference run)

`impact_core::run` over a buy-and-hold strategy that fills one order per bar
against a five-level book, criterion, release build. Throughput is bars/second.

| Book model       | 1,000 bars      | 10,000 bars     |
|------------------|-----------------|-----------------|
| `orderbook_walk` | ~1.85 M bars/s  | ~1.63 M bars/s  |
| `linear_impact`  | ~1.85 M bars/s  | ~1.63 M bars/s  |
| `square_root`    | ~1.84 M bars/s  | ~1.62 M bars/s  |

The three fill models run within noise of each other: the cost is dominated by
the inherited signal/portfolio/metrics loop, not by the walk itself, so measuring
market impact is effectively free over a naive backtest. Absolute numbers depend
on the host; treat them as an order of magnitude (single-digit microseconds per
bar) and reproduce locally for your hardware.
