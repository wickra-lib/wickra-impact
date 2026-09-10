<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Impact — the backtester that knows you would have moved the market" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-impact)
[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/ci.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/ci.yml)
[![CodeQL](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/codeql.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/codeql.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-impact)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/release.svg)](https://github.com/wickra-lib/wickra-impact/releases/latest)
[![crates.io](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/crates.svg)](https://crates.io/crates/wickra-impact)
[![PyPI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/pypi.svg)](https://pypi.org/project/wickra-impact/)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/npm.svg)](https://www.npmjs.com/package/wickra-impact)
[![NuGet](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/nuget.svg)](https://www.nuget.org/packages/Wickra.Impact)
[![Maven Central](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/maven.svg)](https://central.sonatype.com/artifact/org.wickra/wickra-impact)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-impact-go)
[![R-universe](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/r-universe.svg)](https://wickra-lib.r-universe.dev)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/license.svg)](#license)
[![OpenSSF Scorecard](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/scorecard.svg)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-impact)
[![OpenSSF Best Practices](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/best-practices.svg)](https://www.bestpractices.dev)
[![Build provenance](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/provenance.svg)](https://github.com/wickra-lib/wickra-impact/attestations)
[![Docs](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/docs.svg)](https://wickra.org)
[![Verified across 10 languages](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/verified.svg)](golden/)

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

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib):** the same
> data-driven core and ten-language binding surface also power
> [wickra-backtest](https://github.com/wickra-lib/wickra-backtest),
> [wickra-proof](https://github.com/wickra-lib/wickra-proof),
> [wickra-verify](https://github.com/wickra-lib/wickra-verify) and 20 more — see
> [the full list](https://github.com/wickra-lib).

> **Status:** early development (0.1.0, unreleased). The fill engine, the
> reference CLI, the ten-language binding surface, the golden corpus and the full
> CI matrix are all in place; the first published release is still pending.

```rust
use impact_core::{run, ImpactSpec, RunData};

// The moat: walk the order the strategy sent across the real historical book
// and report the price it actually paid, not the one it hoped for.
let spec: ImpactSpec = serde_json::from_str(r#"{
    "strategy": { … },
    "book_model": {"kind": "orderbook_walk"},
    "participation_cap": 1.0,
    "latency_ms": 0
}"#)?;

let report = run(&data, &spec)?;
println!("{} bps average slippage", report.impact_stats.avg_slippage_bps);
```

`orderbook_walk` needs a book, and refuses a run whose data carries none rather
than falling through to a no-op that would report zero slippage. The analytic
models (`linear_impact`, `square_root`) need no book and say so.

## Documentation

- [ARCHITECTURE](docs/ARCHITECTURE.md) — the crates, the inheritance boundary and the JSON-over-C-ABI surface.
- [IMPACT_MODELS](docs/IMPACT_MODELS.md) — the three fill models and their formulas.
- [BOOK_WALK](docs/BOOK_WALK.md) — why every other backtest lies, worked through the `thin_book` example.
- [LATENCY](docs/LATENCY.md) — how a signalled order maps to the book snapshot it fills against.
- [INHERITANCE](docs/INHERITANCE.md) — what IMPACT inherits from `wickra-backtest` unchanged and what it overrides.
- [Cookbook](docs/Cookbook.md) — recipes for common runs.

## How it works

A run request carries the strategy spec, the candle series and — what Impact makes
mandatory — the recorded L2 order books. Where a plain backtest derives a fill
price from a single number, Impact walks the order across the book's price levels,
consuming size at each until the order is filled, and reports the size-weighted
average price. The difference between that and the naive fill is the market impact
your strategy actually paid.

## Quickstart

Run the `thin_book` worked example — a buy-and-hold order that lifts a thin ask
ladder — through the CLI:

```bash
cargo run -p impact-cli -- --request examples/data/requests/thin_book.json
```

```
market impact
=============
avg slippage           44.0000 bps
liquidity consumed      1004.40
partial fills                 0
```

The order for 10 units fills `3 @ 100.1 + 3 @ 100.3 + 4 @ 100.8`, a VWAP of
`100.44` — **44 bps** above the `100.1` inside price a naive backtest would have
charged. Every language binding reproduces that report byte-for-byte.

## ImpactSpec and the book models

An `ImpactSpec` embeds a `wickra-backtest` `StrategySpec` and adds the fill model:

```json
{
  "strategy": { "...": "a wickra-backtest StrategySpec" },
  "book_model": { "kind": "orderbook_walk" },
  "participation_cap": 1.0,
  "latency_ms": 0
}
```

- **`orderbook_walk`** — the moat: consume the real L2 book level by level.
- **`linear_impact` / `square_root`** — analytic curves (`ref·(1 ± coef·q/depth)`
  and `ref·(1 ± coef·√(q/depth))`) for when a full book is not available.
- **`participation_cap`** bounds the fraction of a bar's liquidity one order may
  take; the remainder is a partial fill. **`latency_ms`** shifts which book
  snapshot a signalled order fills against.

See [IMPACT_MODELS](docs/IMPACT_MODELS.md) for the formulas and
[BOOK_WALK](docs/BOOK_WALK.md) for the walk in detail.

## Use in any language

The same handle + `command_json` + `version` surface ships for Rust, Python,
Node.js, WASM, and — over a C ABI hub — C, C++, C#, Go, Java and R. Each binding
forwards the command string verbatim, so the report they return is identical.

## Building everything from source

```bash
cargo build --workspace --all-features                 # Rust core + CLI + C ABI
(cd bindings/python && maturin develop --release)      # Python
(cd bindings/node   && npm ci && npm run build)        # Node
(cd bindings/wasm   && wasm-pack build --target web)   # WASM
(cd bindings/csharp && dotnet build)                   # C#
(cd bindings/go     && go build ./...)                 # Go
(cd bindings/java   && mvn -q package)                 # Java
R CMD INSTALL bindings/r                               # R
```

The C-ABI consumers (C/C++, C#, Go, Java, R) need the C ABI library first —
`cargo build --release -p wickra-impact-c` — on the loader path.

## Project layout

```
crates/impact-core     the fill engine: book walk, impact models, latency
crates/impact-cli      the reference `wickra-impact` binary
crates/impact-bench    criterion benchmarks
bindings/              the ten language surfaces over one C ABI hub
golden/                the cross-language corpus: specs, data, blessed reports
examples/              one runnable example per language
```

## Testing

```bash
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

Every binding replays the same golden runs from [`golden/`](golden/) and must
produce the identical bytes; that corpus is the cross-language contract, not a
per-language approximation. `python scripts/check_binding_surface.py` asserts the
ten surfaces stayed in step.

## Requirements

- **Rust 1.86+** — the workspace MSRV; the Node binding needs **Rust 1.88**.
- **Python 3.9+** — the Python binding.
- **Node 22+** — the Node binding.
- **Go 1.23+** — the Go binding.
- **Java 22+** — the Java binding.
- **R 2.10+** — the R package.
- **.NET 8+** — the C# binding.
- A **C11 / C++17** compiler with CMake for the C and C++ examples.

Impact depends on `wickra-core` for the indicator types, `wickra-backtest` for
the engine it inherits, and `wickra-exchange` for historical L2 books behind the
`live` feature. All three come from crates.io.

## Benchmarks

The order-book walk resolves roughly **1.8 million bars per second** (one signal
plus one book-walk fill per bar); measuring market impact is effectively free over
a naive backtest. See [BENCHMARKS.md](BENCHMARKS.md) and reproduce with
`cargo bench -p impact-bench`.

## Security

See [SECURITY.md](SECURITY.md) and [THREAT_MODEL.md](THREAT_MODEL.md). Impact reads
recorded market data and strategy specs only — no keys, no order placement.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Ecosystem

Part of the [Wickra](https://github.com/wickra-lib/wickra) family — each one a
data-driven core with a CLI and the same ten-language binding surface:

- [**wickra**](https://github.com/wickra-lib/wickra) — main library (Rust core + Python / Node.js / WASM bindings + a C ABI for C / C++ / C# / Go / Java / R)
- [**wickra-playground**](https://github.com/wickra-lib/wickra-playground) — a polyglot strategy playground: one StrategySpec live side by side in Python, Rust, JS and Go, entirely in the browser
- [**wickra-exchange**](https://github.com/wickra-lib/wickra-exchange) — unified market-data + execution across ten crypto exchanges
- [**wickra-backtest**](https://github.com/wickra-lib/wickra-backtest) — event-driven backtester over the Wickra core
- [**wickra-terminal**](https://github.com/wickra-lib/wickra-terminal) — the trading terminal: a TUI and a browser renderer over the stack
- [**wickra-xray**](https://github.com/wickra-lib/wickra-xray) — market-microstructure explorer: footprint, order-book heatmap, liquidation map, funding/OI divergence
- [**wickra-radar**](https://github.com/wickra-lib/wickra-radar) — perp-universe alert radar: OI delta, funding flip, book imbalance, liquidation clusters, OI/price divergence
- [**wickra-copilot**](https://github.com/wickra-lib/wickra-copilot) — local market copilot grounded in real order-book, liquidation and funding microstructure
- [**wickra-shazam**](https://github.com/wickra-lib/wickra-shazam) — match an asset's current microstructure fingerprint against its entire history
- [**wickra-benchmark**](https://github.com/wickra-lib/wickra-benchmark) — reproducible, golden-verified benchmark suite — recompute any (strategy, dataset, report) in ten languages and confirm it byte-for-byte
- [**wickra-strategy-ci**](https://github.com/wickra-lib/wickra-strategy-ci) — Jest for trading strategies: golden-pin the report, catch regressions in CI, property-test against fuzzed data
- [**wickra-verify**](https://github.com/wickra-lib/wickra-verify) — confirm or refute a claimed backtest report against its strategy and data, in ten languages
- [**wickra-proof**](https://github.com/wickra-lib/wickra-proof) — Proof-of-Backtest: deterministic (spec, data) → report + blake3 hash, recomputable byte-for-byte in ten languages
- [**wickra-zk**](https://github.com/wickra-lib/wickra-zk) — prove a backtest zero-knowledge — on-chain-verifiable performance without revealing the data or the strategy
- [**wickra-darwin**](https://github.com/wickra-lib/wickra-darwin) — evolutionary strategy search at millions of backtests per second, mutating and crossing JSON specs across the 514-indicator space
- [**wickra-gym**](https://github.com/wickra-lib/wickra-gym) — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for deterministic RL rollouts
- [**wickra-feature-store**](https://github.com/wickra-lib/wickra-feature-store) — OHLCV and microstructure streams into ML-ready feature matrices over 514 O(1) streaming indicators
- [**wickra-genome**](https://github.com/wickra-lib/wickra-genome) — a vector database of the whole market: every asset a 514-dim live vector, for similarity search, clustering and anomaly detection
- [**wickra-timemachine**](https://github.com/wickra-lib/wickra-timemachine) — scrub the whole market like a video — every symbol, full order book, rewound to any moment via deterministic re-fold
- [**wickra-synth**](https://github.com/wickra-lib/wickra-synth) — deterministic synthetic market microstructure: OHLCV, order book, trades and funding from a single seed
- [**wickra-compile**](https://github.com/wickra-lib/wickra-compile) — compile a strategy spec into a standalone deployable: a WASM module, a self-contained binary, or a `no_std` artifact
- [**wickra-embed**](https://github.com/wickra-lib/wickra-embed) — allocation-free, `no_std` streaming indicators for bare-metal and HFT, byte-for-byte identical to the core
- [**wickra-pico**](https://github.com/wickra-lib/wickra-pico) — the O(1) indicator core running bare-metal on a $5 Raspberry Pi Pico — the LED blinks on the EMA cross

The screener's own guides live in [`docs/`](docs/) beside the code; its site,
with the in-browser demo and the benchmark figures, is at
[screener.wickra.org](https://screener.wickra.org). The indicator library's
reference is at [docs.wickra.org](https://docs.wickra.org) and the org landing
page at [wickra.org](https://wickra.org).

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

<p align="center">
  <a href="https://github.com/wickra-lib/wickra-impact">
    <img alt="GitHub stars" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/stars.svg">
  </a>
  <a href="https://github.com/wickra-lib/wickra-impact/network/members">
    <img alt="GitHub forks" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/forks.svg">
  </a>
  <a href="https://github.com/wickra-lib/wickra-impact/issues">
    <img alt="GitHub issues" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/issues.svg">
  </a>
</p>

<p align="center">
  Built on <a href="https://github.com/wickra-lib/wickra">Wickra</a>. If it saved you time, the cheapest way to say thanks is to ⭐ the repo.
</p>

<p align="center">
  <img alt="wickra-impact star history" width="640"
       src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/star-history.svg">
</p>

## Disclaimer

Wickra Impact is a research and backtesting tool. It measures the slippage an
order would have paid against recorded market data; it does not place orders,
and a measurement over history is not a prediction about the future. Nothing
here is financial advice. Trading carries risk, including the loss of the
capital committed. Use it at your own risk.
