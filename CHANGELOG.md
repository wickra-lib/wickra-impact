# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **`CITATION.cff` described the wrong project.** The abstract and the keyword
  list were the feature store's, describing a feature matrix for a market-impact
  model. `CITATION.cff` is what GitHub's citation box and Zenodo quote back at a
  reader as the project's own words, so it is the one file where a wrong
  description is the project saying it.

- **The Ecosystem section repeated two claims their own repositories had already
  corrected**: DARWIN at "millions of backtests per second" across "the
  514-indicator space", where its benchmark says hundreds of thousands over the
  registry, and GENOME as "a 514-dim live vector", where the dimension is
  whatever the spec's feature list names.

- **CONTRIBUTING.md described a different repository** — feature kinds, label
  kinds, `docs/FEATURES.md`, `docs/LABELS.md`, all of it
  wickra-feature-store's. This is what has been failing the link check on
  `main`.

- **`CMAKE_CXX_STANDARD` asked for C++14** while the C++ hull requires C++17.
  Nothing compiled it, so nothing found out.

### Added

- **Book-refusal tests in every binding.** Python, Node, Go, Java, C#, R, WASM
  and C each check that `orderbook_walk` refuses a run whose data carries no
  book, that it measures 44 bps when the book is there, that an analytic model
  needs none, and that the run is reproducible. That refusal is the one way this
  engine's claim can fail quietly: a binding that swallowed it would report zero
  slippage from the model whose entire purpose is to find some.

- **A core test for the same refusal**, which the Rust suite did not have
  either.

- **A golden test for the C binding**, which had none: all six committed specs,
  byte-identical.

- The blueprint scaffold: `LICENSES/`, `docs/README.md`, the five long-form
  issue templates, the CodeQL config, the actionlint and CodSpeed workflows, the
  five check scripts, a C++ hull, licence copies in every published crate and
  npm package, and a WASM example.

- CI gains `osv`, `links`, `binding-surface`, `semver`, `fuzz-smoke`,
  `examples` and `python-wheel-container-smoke`; the release pipeline gains the
  `gate` and `guard` jobs, provenance over the nupkg, jar and C ABI archives, a
  Maven artifact on the release page, and a Go mirror that builds before it
  publishes.

### Changed

- **The family pins move to the published releases.** `wickra-backtest` and
  `wickra-exchange` come from crates.io rather than git revs, and `wickra-core`
  / `wickra-data` rise from 0.9 to 1.0, so the tree carries one set of indicator
  types rather than two that share none.

- **`BacktestReport` gained `symbol` and `timeframe`** with that bump, so the
  blessed reports were re-blessed. Every number in them is unchanged — only the
  two new fields differ — which is what says the fill engine itself did not
  move.

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
