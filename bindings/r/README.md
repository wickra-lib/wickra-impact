# Wickra Impact — R

R bindings for the Wickra Impact market-impact backtester over its C ABI hub, via
`.Call`. A backtest is built from a spec JSON and driven over a JSON boundary, so
the result is byte-identical to every other Wickra Impact binding.

## Build & test

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

## Usage

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

## Surface

- **`wkimpact_new(spec_json)`** — build a backtest handle from a spec JSON (an
  external pointer; `"{}"` defers configuration to a later `set_spec`).
- **`wkimpact_command(impact, cmd_json)`** — apply a command envelope
  (`{"cmd":"...", ...}`) and return the response JSON. Commands: `set_spec`,
  `run`, `version`.
- **`wkimpact_version()`** — the library version.

## Determinism

The fill engine lives only in the Rust core; this binding forwards the command
string verbatim, so a given request produces the byte-identical report here and
in every other binding — the exact cross-language golden invariant.

## See also

- The main project: <https://github.com/wickra-lib/wickra-impact>
- Documentation: <https://wickra.org>

## License

Dual-licensed under either MIT or Apache-2.0, at your option.
