# wickra-impact WASM examples

Browser demos for the `wickra-impact-wasm` binding.

The WASM build carries the whole fill engine with `--no-default-features`: the
book walk runs sequentially rather than in parallel, and byte-for-byte
identically, which is what the golden reports pin down. A spec is data, not code,
so the bytes on this page are the same ones `examples/node/run.js` sends, and the
slippage is the same slippage.

## Build

The module ships as a `wasm-pack` `--target web` bundle. Build it once from the
repository root:

```bash
wasm-pack build bindings/wasm --target web --release
```

That writes `bindings/wasm/pkg/` with the `.wasm` binary, the JS loader and the
type declarations the page imports.

## Run

The page loads its module over `http://`, not `file://`, because ES module
imports and `WebAssembly.instantiateStreaming` both need a real origin. Serve the
repository root:

```bash
python -m http.server 8000
```

Then open `http://localhost:8000/examples/wasm/slippage.html`.

## Pages

| Page | What it does |
|------|--------------|
| `slippage.html` | Sends a ten-unit market order into a book whose top level holds three, under each fill model in turn, and shows what each says the order paid. `orderbook_walk` walks all three ask levels; the analytic curves price from a formula; a plain backtest would take the top of book and report no slippage at all. The page counterpart of `examples/node/run.js`. |

## See also

- [examples/README.md](../README.md) — the same run in every other language.
- [bindings/wasm/README.md](../../bindings/wasm/README.md) — the binding itself.
