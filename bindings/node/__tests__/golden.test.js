"use strict";

// Determinism: the same request yields the byte-identical report string. The
// full cross-language golden (asserting the response equals a blessed
// golden/expected file) lands with the golden corpus in P-IMP-4; here we pin the
// core invariant that a run is byte-reproducible, which every binding must
// preserve by forwarding the command string verbatim.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Impact } = require("../index.js");

const GOLDEN = path.resolve(__dirname, "..", "..", "..", "golden");

const SPEC = {
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
};

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

test("the same request yields the byte-identical report string", () => {
  const cmd = JSON.stringify({ cmd: "run", data: DATA });
  const a = new Impact(JSON.stringify(SPEC)).command(cmd);
  const b = new Impact(JSON.stringify(SPEC)).command(cmd);
  assert.strictEqual(a, b);
});

test("the report carries the measured impact", () => {
  const cmd = JSON.stringify({ cmd: "run", data: DATA });
  const report = JSON.parse(new Impact(JSON.stringify(SPEC)).command(cmd));
  assert.strictEqual(report.impact_stats.avg_slippage_bps, 44.0);
});

// The cross-language golden: for every spec in the shared corpus, the Node
// binding's response string must equal `golden/expected/<name>.json`
// byte-for-byte — the same fixture the Rust core, the CLI and every other binding
// reproduce.
for (const specFile of fs.readdirSync(path.join(GOLDEN, "specs")).sort()) {
  const name = path.basename(specFile, ".json");
  test(`cross-language golden: ${name}`, () => {
    const spec = fs.readFileSync(path.join(GOLDEN, "specs", specFile), "utf8");
    const data = JSON.parse(fs.readFileSync(path.join(GOLDEN, "data", `${name}.json`), "utf8"));
    const expected = fs
      .readFileSync(path.join(GOLDEN, "expected", `${name}.json`), "utf8")
      .trimEnd();
    const got = new Impact(spec).command(JSON.stringify({ cmd: "run", data }));
    assert.strictEqual(got, expected, `golden mismatch for ${name}`);
  });
}
