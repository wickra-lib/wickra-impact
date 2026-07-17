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

[Unreleased]: https://github.com/wickra-lib/wickra-impact/commits/main
