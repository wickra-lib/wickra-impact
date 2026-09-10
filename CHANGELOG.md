# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **The `latency` golden case proved nothing.** It carried the same two-bar
  dataset as every other spec, and `fill_timing: next_open` plus `ceil(1000 /
  3_600_000)` = one bar put the fill on bar 2 of a two-bar history -- so the
  order was cancelled and the blessed report was all zeros: no trades, no
  slippage, no liquidity consumed. A case named `latency` was pinning "the
  dataset is too short". It now runs over four bars whose books step from 100.10
  to 101.50, so the entry price is what shows the latency moved the fill, and
  two tests pin both halves of the rule: a latency spanning a bar fills against
  the later book, and one running past the recorded history cancels.

- **`cargo-deny` was set to warn about duplicated crates, so it noted that split
  and moved on.** It is an error now. Only four duplicates exist across this
  workspace and each is a crate part-way through a major release reached through
  two ecosystems; they are skipped by name with the reason recorded, so a fifth
  still fails. Verified by putting 6.1.3 back and watching the check fail on
  `convert_case` before the compiler ever ran.

- **`actionlint` failed on five shell constructs the screener had already
  fixed.** `a && b || c` is not if-then-else -- when the publish succeeded but
  the echo failed, the fallback branch ran and reported "already published";
  `local pkg=$(basename …)` and `export PATH="$(cygpath …)"` hide the command's
  exit status behind `local`/`export`; and an asset count taken from `ls` breaks
  on a filename containing a newline. The runner-label config the linter needs
  for `windows-11-arm` was missing too.

- **A yanked crate was in the lockfile.** `wnaf` 0.14.0, reached through `p256`
  -> `wickra-exchange-core`, was yanked from crates.io; 0.14.1 is not.

- **Three steps of the `examples` job ran a file that is not here.** Python,
  Node.js and R each invoked `examples/<lang>/scan.*` -- the screener's file
  name, left over from the port -- so they died on a missing file before
  reaching any assertion.

- **Every language step asserted `"symbol":"BBB"`**, a line from the screener's
  scan report that no example here prints. Each step now matches a string its
  own example emits, read off the format string rather than guessed: the
  assertions were checked against a real run of each example.

- **The Rust example's lockfile pinned the pre-migration engine.** It still held
  `wickra-core` 0.9.9 and `wickra-backtest` 0.1.0 while the workspace declares
  1.0 and 0.1.4, so cargo silently repaired the lock on every build and the
  example was the one reach measured against a different engine than the rest.

- **The C++ example now goes through the C++ hull.** It called the C functions
  directly and rebuilt the two-call length protocol by hand -- the very thing
  `wickra_impact.hpp` exists to remove -- which left the shipped C++ surface
  built by nothing. Verified by running both: the C and C++ examples print
  byte-identical output.

- **Every C++ hull used the include guard `WICKRA_SCREENER_HPP`.** The C headers
  beside them are guarded correctly; only the `.hpp` files shared one name, so
  including two of the family's headers in the same translation unit dropped the
  second silently. Proven by compiling a file that includes two of them and
  names a class from each: `'Env' is not a member of 'wickra'`. All seven now
  compile standalone and together.

- **The hull's usage example could not run.** It showed a spec shaped
  `{"universe":[...]}` and `{"cmd":"scan"}`, the screener's, which this core
  rejects twice over. It now shows this repository's own spec fields and one of
  its own commands.

- **The release notes named the wrong package.** They told a reader
  `install.packages("wickrafeaturestore")` from r-universe, where this package
  is `wickraimpact`. These notes go out with the GitHub release: a reader
  following them installs a different library.

- **The issue and pull-request templates asked for a `FeatureSpec`**, a type
  this repository does not have, so a contributor was asked to attach something
  that does not exist. `GOVERNANCE.md`, `SUPPORT.md` and `CONTRIBUTING.md`
  carried the same substitution, along with the screener's "condition schema"
  for a core that has no conditions.

- **The R `configure` scripts still defined `wkscreen_download`**, the last
  trace of the screener's prefix — the CI-visible half of which already had to
  be fixed once.

- **Six SHA-pinned actions sat on two lines across the family**, and two of the
  splits were inside this repository. `actions/setup-node` is pinned at the same
  commit everywhere, but some call sites annotated it `# v6.4.0`; GitHub's tag
  list says that commit is **v7.0.0** and v6.4.0 is a different one. Dependabot
  reads that comment to decide what to bump, so a wrong one misdirects the tool
  meant to keep the pin current. `Swatinem/rust-cache` ran at two commits at
  once, the older behind a floating `# v2`. Every pin now matches what the
  sibling repositories run, each target checked against the upstream tag list.

- **The CI Java example step compiled a file that is not there.** The `examples`
  job was ported from the screener, whose Java example is a single
  `examples/java/Scan.java` built with `javac`. This repository ships a Maven
  project instead, so the step compiled a missing file and then asserted on
  output the example never prints. It now builds the binding into the local
  repository and runs the example through `mvn exec:exec`, the way the example's
  own javadoc documents -- verified by running it.

- **The `examples` job installed a lockfile that is not here.** It names
  `.github/requirements/ci-dev-py3.txt`, and so does `scripts/update-lockfiles.sh`,
  but the directory held a single `ci-dev.txt` that nothing referenced. The split
  is not cosmetic: the Python matrix includes 3.9, and that single lock pinned
  `pytest==9.1.1` and `iniconfig==2.3.0`, both of which declare
  requires-python >= 3.10.

- **The `python` job installed unpinned.** `pip install maturin pytest` is a
  fetch of whatever the index serves that minute -- the exact thing the locked
  file exists to prevent. It now installs the hash-locked row for its
  interpreter, and the advisory the 3.9 pin sits inside is recorded with its
  reason in `osv-scanner.toml`.

- **Dependabot watched directories that do not exist**, so it reported nothing
  and the silence read as calm. `nuget` pointed at `WickraCompile.Tests`, a
  project name from another repository; `pip` did not cover
  `/.github/requirements` and `npm` did not cover `/examples/node`.

- **The workspace's own core was pinned as a range.** `impact-core` was named
  six times as `version = "0.1"` -- a caret range -- and the root manifest
  carried no `[workspace.dependencies]` entry for it at all. A published
  `impact-cli` 0.1.0 would have accepted `impact-core` 0.1.99, a crate resolving
  against a core it was never built against, in a workspace whose whole point is
  that the pieces move together. It also hid the line from `bump_version.py` and
  `check_version_sync.py`, both of which look for the exact version.

- **`release.yml` overwrote the binding READMEs before packing.** Three steps
  copied the root README over `bindings/python/README.md` (wheel and sdist) and
  `bindings/node/README.md`. They date from when the bindings had no README of
  their own; they do now, one per registry, and `check_readme_links.py` exists to
  keep their links absolute because a relative link is dead on PyPI and npm. The
  copy threw that away and shipped the root README, whose links are relative by
  design. The remaining relative links in the C, C#, Go and WASM READMEs are
  absolute now.

- **The Python wheel would have shipped without its licence texts.**
  `bindings/python/` carried neither `LICENSE-MIT` nor `LICENSE-APACHE`, so
  maturin had nothing to include, while every crate and the release archive
  carry both.

- **`SECURITY.md` named a support policy for releases that do not exist yet.**
  It promised fixes for "the latest `0.x` release line" where there is no
  released line; it now says plainly that nothing is published and names `0.1.0`
  as the first version that will be.

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
