# Wickra Impact — WASM

WebAssembly bindings for the Wickra Impact market-impact backtester, compiled
from Rust with [wasm-bindgen](https://wasm-bindgen.github.io/wasm-bindgen/). An
`Impact` is built from a spec JSON and driven by command JSONs over a JSON
boundary, so a browser front-end runs against the exact same core as every other
Wickra Impact binding.

## Build

```bash
wasm-pack build --target web      # for a browser bundler
wasm-pack build --target nodejs   # for node:test / Node.js
```

The output lands in `pkg/`.

## Usage

```js
import init, { Impact } from "./pkg/wickra_impact_wasm.js";

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

## Determinism

The fill engine runs single-threaded here — no rayon thread pool in a browser
sandbox — which is byte-identical to the native, parallel-capable run. A given
request produces the byte-identical report here and in every other binding: the
exact cross-language golden invariant.

## See also

- The main project: <https://github.com/wickra-lib/wickra-impact>
- Documentation: <https://wickra.org>

## License

Dual-licensed under either [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
