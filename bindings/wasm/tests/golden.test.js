"use strict";

// Golden test over the wasm-pack (nodejs target) output: the WebAssembly build
// measures market impact byte-identically to the native run — the
// single-threaded fill engine in the browser sandbox reproduces the same request
// exactly. Skips cleanly when `pkg/` has not been built yet
// (`wasm-pack build --target nodejs`).

const { test } = require("node:test");
const assert = require("node:assert");
const path = require("node:path");

let wasm = null;
try {
  wasm = require(path.resolve(__dirname, "..", "pkg", "wickra_impact_wasm.js"));
} catch {
  wasm = null;
}

const SPEC = JSON.stringify({
  strategy: {
    spec_version: 1,
    symbol: "IMPACT",
    timeframe: "1h",
    indicators: {},
    entry: { ge: [{ price: "close" }, 0] },
    exit: { in_position: true },
    sizing: { type: "fixed_qty", qty: 10.0 },
    execution: { order_type: "market", fill_timing: "next_open" },
  },
  book_model: { kind: "orderbook_walk" },
  participation_cap: 1.0,
  latency_ms: 0,
});

const DATA = {
  candles: [
    { time: 0, open: 100, high: 100, low: 100, close: 100, volume: 1000 },
    { time: 3600, open: 100, high: 103, low: 100, close: 102, volume: 1000 },
  ],
  books: [
    { bids: [{ price: 99.9, size: 100 }], asks: [{ price: 100.1, size: 100 }] },
    {
      bids: [{ price: 99.9, size: 100 }],
      asks: [
        { price: 100.1, size: 3 },
        { price: 100.3, size: 3 },
        { price: 100.8, size: 4 },
      ],
    },
  ],
};

function runCmd() {
  return JSON.stringify({ cmd: "run", data: DATA });
}

test("wasm build present or skipped", (t) => {
  if (!wasm) t.skip("run `wasm-pack build --target nodejs` first");
});

if (wasm) {
  test("wasm run measures the expected market impact", () => {
    const out = JSON.parse(new wasm.Impact(SPEC).command(runCmd()));
    assert.strictEqual(out.impact_stats.avg_slippage_bps, 44.0);
    assert.strictEqual(out.report.trades[0].entry_price, 100.44);
  });

  test("wasm run is byte-identical across calls", () => {
    const a = new wasm.Impact(SPEC).command(runCmd());
    const b = new wasm.Impact(SPEC).command(runCmd());
    assert.strictEqual(a, b);
  });

  test("wasm version matches the module export", () => {
    assert.strictEqual(new wasm.Impact(SPEC).version(), wasm.version());
  });

  test("wasm throws on an invalid spec", () => {
    assert.throws(() => new wasm.Impact("{ not valid json"));
  });
}
