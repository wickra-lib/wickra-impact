"use strict";

const { test } = require("node:test");
const assert = require("node:assert");
const { Impact } = require("../index.js");

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

function run(impact) {
  return JSON.parse(impact.command(JSON.stringify({ cmd: "run", data: DATA })));
}

test("run returns a report with the measured market impact", () => {
  const report = run(new Impact(JSON.stringify(SPEC)));
  assert.strictEqual(report.impact_stats.avg_slippage_bps, 44.0);
  assert.strictEqual(report.report.trades[0].entry_price, 100.44);
});

test("deterministic across instances", () => {
  const a = run(new Impact(JSON.stringify(SPEC)));
  const b = run(new Impact(JSON.stringify(SPEC)));
  assert.deepStrictEqual(a, b);
});

test("set_spec then run", () => {
  const impact = new Impact("{}");
  const ok = JSON.parse(impact.command(JSON.stringify({ cmd: "set_spec", spec: SPEC })));
  assert.strictEqual(ok.ok, true);
  assert.ok("impact_stats" in run(impact));
});

test("version", () => {
  assert.strictEqual(typeof new Impact("{}").version(), "string");
});
