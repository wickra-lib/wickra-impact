# Benchmarks

The headline figure is **fills per second** — the rate at which the order-book
walk resolves a market order against a recorded L2 book, level by level, into a
size-weighted fill price. This is the work IMPACT adds on top of the inherited
`wickra-backtest` engine.

Reproduce with:

```bash
cargo bench -p impact-bench
```

_Placeholder — replaced with measured figures during the test-and-bench phase._
