"use strict";

// The book the walk needs, through the JSON command boundary.
//
// There is no streaming half in IMPACT: a run is a batch computation. What stands
// in its place is the property the whole repository exists for -- that slippage is
// measured rather than guessed -- and the one way it can fail quietly.
//
// `orderbook_walk` needs a book. Without one the core refuses the run. A binding
// that swallowed that refusal would hand back a report showing zero slippage from
// the model whose entire purpose is to find some, and nothing downstream could
// tell the difference between "no impact" and "no book".

const { test } = require("node:test");
const assert = require("node:assert");
const { Impact } = require("../index.js");

const STRATEGY = {
  spec_version: 1,
  symbol: "IMPACT",
  timeframe: "1h",
  indicators: {},
  entry: { ge: [{ price: "close" }, 0] },
  exit: { in_position: true },
  sizing: { type: "fixed_qty", qty: 10.0 },
  execution: { order_type: "market", fill_timing: "next_open" },
};

const CANDLES = [
  { time: 0, open: 100, high: 100, low: 100, close: 100, volume: 1000 },
  { time: 3600, open: 100, high: 103, low: 100, close: 102, volume: 1000 },
];

const BOOKS = [
  { bids: [{ price: 99.9, size: 100 }], asks: [{ price: 100.1, size: 100 }] },
  {
    bids: [{ price: 99.9, size: 100 }],
    asks: [
      { price: 100.1, size: 3 },
      { price: 100.3, size: 3 },
      { price: 100.8, size: 4 },
    ],
  },
];

const spec = (model) => JSON.stringify({
  strategy: STRATEGY,
  book_model: model,
  participation_cap: 1.0,
  latency_ms: 0,
});

const run = (model, data) =>
  new Impact(spec(model)).command(JSON.stringify({ cmd: "run", data }));

test("the walk refuses a run with no book", () => {
  const response = run({ kind: "orderbook_walk" }, { candles: CANDLES });
  assert.match(response, /book/, response);
  assert.doesNotMatch(
    response,
    /"impact_stats"/,
    "a walk with no book must be refused, not answered with zero slippage",
  );
});

test("the walk measures slippage when the book is there", () => {
  const report = JSON.parse(
    run({ kind: "orderbook_walk" }, { candles: CANDLES, books: BOOKS }),
  );
  // Walking three ask levels for ten units pays 100.44, not the 100.10 top of
  // book a naive fill would take.
  assert.strictEqual(report.impact_stats.avg_slippage_bps, 44.0);
  assert.strictEqual(report.report.trades[0].entry_price, 100.44);
});

test("an analytic model needs no book", () => {
  const report = JSON.parse(
    run({ kind: "linear_impact", coef: 0.1 }, { candles: CANDLES }),
  );
  assert.ok(report.impact_stats, "linear_impact prices from a curve, not a book");
});

test("the batch run is reproducible", () => {
  const data = { candles: CANDLES, books: BOOKS };
  assert.strictEqual(
    run({ kind: "orderbook_walk" }, data),
    run({ kind: "orderbook_walk" }, data),
  );
});
