<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Impact — the backtester that knows you would have moved the market" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/ci.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-impact)
[![r-universe](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/r-universe.svg)](https://wickra-lib.r-universe.dev)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/license.svg)](https://github.com/wickra-lib/wickra-impact#license)

# Wickra Impact — R

---

**Part of the [Wickra ecosystem](#ecosystem): — for R. `install.packages("wickraimpact", repos = "https://wickra-lib.r-universe.dev")` — over the C ABI via `.Call`, prebuilt library fetched on install.**

R bindings for the Wickra Impact market-impact backtester over its C ABI hub, via
`.Call`. A backtest is built from a spec JSON and driven over a JSON boundary, so
the result is byte-identical to every other Wickra Impact binding.

## Install

From r-universe:

```r
install.packages("wickraimpact", repos = "https://wickra-lib.r-universe.dev")
```

The package's `configure` downloads the prebuilt C ABI library for this exact
version from the GitHub release and bundles it, so an ordinary install needs
nothing but a C toolchain (Rtools on Windows) for the thin `.Call` glue layer. To
build against a local checkout instead, point it at the header and library with
the environment variables below.

### Building from this repository (contributors)

The C ABI header and shared library are provided out-of-tree through two
environment variables (set by CI / the installer):

```bash
export WKIMPACT_INC=/path/to/bindings/c/include   # the header dir
export WKIMPACT_LIB=/path/to/target/release       # the library dir
R CMD INSTALL bindings/r
Rscript bindings/r/tests/run_tests.R
```

At run time the loader must find the shared library on `LD_LIBRARY_PATH`
(Linux), `DYLD_LIBRARY_PATH` (macOS) or `PATH` (Windows).

## Quick start

```r
library(wickraimpact)

spec <- paste0(
  '{"strategy":{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",',
  '"indicators":{},"entry":{"ge":[{"price":"close"},0]},',
  '"exit":{"in_position":true},"sizing":{"type":"fixed_qty","qty":10.0},',
  '"execution":{"order_type":"market","fill_timing":"next_open"}},',
  '"book_model":{"kind":"orderbook_walk"},',
  '"participation_cap":1.0,"latency_ms":0}'
)

impact <- wkimpact_new(spec)
response <- wkimpact_command(impact, paste0('{"cmd":"run","data":', data, "}"))
cat(response)  # the report carries the market impact a naive backtest hides
```

### Surface

- **`wkimpact_new(spec_json)`** — build a backtest handle from a spec JSON (an
  external pointer; `"{}"` defers configuration to a later `set_spec`).
- **`wkimpact_command(impact, cmd_json)`** — apply a command envelope
  (`{"cmd":"...", ...}`) and return the response JSON. Commands: `set_spec`,
  `run`, `version`.
- **`wkimpact_version()`** — the library version.

### Determinism

The fill engine lives only in the Rust core; this binding forwards the command
string verbatim, so a given request produces the byte-identical report here and
in every other binding — the exact cross-language golden invariant.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of R's native `.Call` interface over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-impact/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-impact>
- **Docs** (guides, spec reference, cookbook): <https://impact.wickra.org>
- **Runnable example:** [`examples/r/`](https://github.com/wickra-lib/wickra-impact/tree/main/examples/r)

- The main project: <https://github.com/wickra-lib/wickra-impact>
- Documentation: <https://wickra.org>

Wickra Impact ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-impact/blob/main/SECURITY.md>.

## Disclaimer

Wickra Impact is a research and backtesting tool. It measures the slippage an
order would have paid against recorded market data; it does not place orders,
and a measurement over history is not a prediction about the future. Nothing
here is financial advice. Trading carries risk, including the loss of the
capital committed. Use it at your own risk.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-MIT) at your option.
