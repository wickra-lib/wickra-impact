# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Repository scaffold: governance, supply-chain configuration (`deny.toml`,
  `lychee.toml`, `osv-scanner.toml`, `repo-metadata.toml`), the Rust workspace
  (`impact-core`, `impact-cli`, `impact-bench`) with the language-binding crates,
  and the `wickra-backtest` / `wickra-exchange` git dependencies (the engine
  IMPACT inherits and the historical L2 order books it walks).
- `impact-core`: the market-impact engine — the `BookModel` fill engine
  (order-book walk, linear and square-root impact), the `ImpactSpec` envelope over
  an embedded `wickra-backtest` strategy, latency-to-snapshot mapping, and the
  `run` loop that reconstructs the inherited `BacktestReport` with real order-book
  fills plus an `ImpactStats` block, exposed over the `command_json` boundary
  (`Impact`). A fidelity test pins that a zero-impact run reproduces the engine's
  own result.
- `wickra-impact` CLI over the core: `--request` (a `{spec, data}` bundle),
  `--spec` + `--data`, or `--stdin`, with `--format text|json`. The text output
  summarises the backtest and the market-impact block (average slippage, liquidity
  consumed, partial fills).
- Ten language bindings over the `command_json` boundary: native Rust, Python
  (PyO3), Node.js (napi-rs) and WASM (wasm-bindgen), plus C, C++, C#, Go, Java and
  R over a C ABI hub. Each forwards the command string verbatim, so every binding
  returns the byte-identical `ImpactReport`.
- The golden corpus (`golden/`): six `spec`×`data` pairs and their blessed
  `expected/` reports covering the book-model matrix (`thin_book`,
  `thin_book_capped`, `deep_book`, `linear_impact`, `square_root`, `latency`),
  generate-once / replay-everywhere.
- Test rigor: byte-golden replay, serde/validation conformance, the walk-vs-naive
  proof, proptest invariants over the fill engine, cargo-fuzz targets
  (`spec_parse`, `book_walk`, `run_batch`, `latency_select`), a criterion bench
  crate, and a cross-language golden guard in the bindings.
- Runnable examples in all ten languages plus a CMake/ctest C/C++ harness, each
  running the `thin_book` request to the same summary.
- CI/CD: the full workflow suite (multi-OS × multi-language matrix, coverage,
  cargo-deny, fuzz-smoke, header-drift, CodeQL, Scorecard, zizmor, links,
  nightly bench) and a tag-gated, USER-GO release pipeline.

[Unreleased]: https://github.com/wickra-lib/wickra-impact/commits/main
