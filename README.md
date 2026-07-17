<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Impact — the backtester that knows you would have moved the market" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-impact)
[![CI](https://github.com/wickra-lib/wickra-impact/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/ci.yml)
[![CodeQL](https://github.com/wickra-lib/wickra-impact/actions/workflows/codeql.yml/badge.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/codeql.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![OpenSSF Scorecard](https://img.shields.io/badge/OpenSSF-Scorecard-3b82f6)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-impact)
[![Deterministic across 10 languages](https://img.shields.io/badge/deterministic%20across-10%20languages-3b82f6)](#use-in-any-language)
[![Docs](https://img.shields.io/badge/docs-wickra.org-3b82f6)](https://wickra.org)

---

# Wickra Impact

**The backtester that knows you would have moved the market — agent-based fills on
the real historical L2 order book, so slippage is measured, not guessed.**

Every ordinary backtest lies: it fills your order at the close, or at a fixed
slippage estimate, as if your size were invisible. Wickra Impact does not. It
walks your order through the actual recorded L2 order book — eating liquidity
level by level — so the fill price is what the market would really have given you,
impact included.

Impact is one library, `impact-core`: it **inherits the `wickra-backtest` engine
1:1** (its `StrategySpec`, `RunRequest` and `BacktestReport`) and replaces **only
the fill stage** with an order-book-walk fill engine. It is usable in **Rust,
Python, Node.js, WASM, C, C++, C#, Go, Java and R** over a JSON-over-C-ABI
boundary (`command_json`), plus a reference CLI.

> **Status:** early scaffold (0.1.0, unreleased). The repository skeleton,
> workspace and governance are in place; the fill engine, the CLI, the ten
> bindings, the golden corpus and CI land in the phases that follow.

## How it works

A run request carries the strategy spec, the candle series and — what Impact makes
mandatory — the recorded L2 order books. Where a plain backtest derives a fill
price from a single number, Impact walks the order across the book's price levels,
consuming size at each until the order is filled, and reports the size-weighted
average price. The difference between that and the naive fill is the market impact
your strategy actually paid.

## Use in any language

The same handle + `command_json` + `version` surface ships for Rust, Python,
Node.js, WASM, and — over a C ABI hub — C, C++, C#, Go, Java and R. Each binding
forwards the command string verbatim, so the report they return is identical.

## Building from source

```bash
cargo build
cargo test
```

## Requirements

- Rust 1.86+ (MSRV). Impact depends on `wickra-core` (crates.io) and, as git
  dependencies, `wickra-backtest` (the engine it inherits) and `wickra-exchange`
  (historical L2 books, behind the `live` feature).

## Security

See [SECURITY.md](SECURITY.md) and [THREAT_MODEL.md](THREAT_MODEL.md). Impact reads
recorded market data and strategy specs only — no keys, no order placement.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
