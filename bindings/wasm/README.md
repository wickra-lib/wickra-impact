<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Impact — the backtester that knows you would have moved the market" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/ci.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-impact)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/npm.svg)](https://www.npmjs.com/package/wickra-impact-wasm)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/license.svg)](https://github.com/wickra-lib/wickra-impact#license)

# Wickra Impact — WASM

---

**Part of the [Wickra ecosystem](#ecosystem): — for WASM. `npm install wickra-impact-wasm` — pure WebAssembly, runs anywhere a modern JS engine does.**

WebAssembly bindings for the Wickra Impact market-impact backtester, compiled
from Rust with [wasm-bindgen](https://wasm-bindgen.github.io/wasm-bindgen/). An
`Impact` is built from a spec JSON and driven by command JSONs over a JSON
boundary, so a browser front-end runs against the exact same core as every other
Wickra Impact binding.

## Install

```bash
npm install wickra-impact-wasm
```

### Building from this repository (contributors)

```bash
wasm-pack build --target web      # for a browser bundler
wasm-pack build --target nodejs   # for node:test / Node.js
```

The output lands in `pkg/`.

## Quick start

```js
import init, { Impact } from "wickra-impact-wasm";

await init();

const spec = JSON.stringify({
  strategy: { spec_version: 1, symbol: "IMPACT", timeframe: "1h",
    indicators: {},
    entry: { ge: [{ price: "close" }, 0] },
    exit: { in_position: true },
    sizing: { type: "fixed_qty", qty: 10.0 },
    execution: { order_type: "market", fill_timing: "next_open" } },
  book_model: { kind: "orderbook_walk" },
  participation_cap: 1.0, latency_ms: 0,
});

const impact = new Impact(spec);
const report = JSON.parse(impact.command(JSON.stringify({ cmd: "run", data })));
console.log(report.impact_stats.avg_slippage_bps); // the impact a naive backtest hides
```

`command` mirrors `Impact::command_json`: the commands are `set_spec`, `run` and
`version`. An invalid spec throws; a command failure throws too.

### Determinism

The fill engine runs single-threaded here — no rayon thread pool in a browser
sandbox — which is byte-identical to the native, parallel-capable run. A given
request produces the byte-identical report here and in every other binding: the
exact cross-language golden invariant.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of wasm-bindgen, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-impact/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-impact>
- **Docs** (guides, spec reference, cookbook): <https://impact.wickra.org>
- **Runnable example:** [`examples/wasm/`](https://github.com/wickra-lib/wickra-impact/tree/main/examples/wasm)

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
