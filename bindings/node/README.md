# Wickra Impact — Node.js

Node.js bindings for the Wickra Impact market-impact backtester, powered by Rust
via [napi-rs](https://napi.rs/): back-test a strategy against the real historical
L2 order book, measuring the slippage every other backtest ignores.

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/wickra-lib/wickra-impact#license)
[![napi-rs](https://img.shields.io/badge/bindings-napi--rs-3b82f6)](https://napi.rs)
[![Docs](https://img.shields.io/badge/docs-wickra.org-3b82f6)](https://wickra.org)

## Install

```bash
npm install wickra-impact
```

## Usage

```js
const { Impact } = require("wickra-impact");

const spec = {
  strategy: { spec_version: 1, symbol: "IMPACT", timeframe: "1h",
    indicators: {},
    entry: { ge: [{ price: "close" }, 0] },
    exit: { in_position: true },
    sizing: { type: "fixed_qty", qty: 10.0 },
    execution: { order_type: "market", fill_timing: "next_open" } },
  book_model: { kind: "orderbook_walk" },
  participation_cap: 1.0, latency_ms: 0,
};

const impact = new Impact(JSON.stringify(spec));
const report = JSON.parse(impact.command(JSON.stringify({ cmd: "run", data })));
console.log(report.impact_stats.avg_slippage_bps); // the impact a naive backtest hides
```

## API

- **`new Impact(specJson)`** — construct a backtest handle from an `ImpactSpec`
  JSON (`"{}"` defers configuration to a later `set_spec`). Throws on an invalid
  spec.
- **`impact.command(cmdJson)`** — apply a command envelope and return the response
  JSON. Commands: `set_spec`, `run`, `version`.
- **`impact.version()`** — the library version.

## Determinism

`command` mirrors the core's `command_json`: the fill engine lives only in the
Rust core and this binding forwards the command string verbatim, so the report is
byte-identical to the CLI and to every other language binding.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-APACHE).
