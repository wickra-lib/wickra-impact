// A runnable Node.js example: back-test a buy-and-hold strategy against a thin
// order book and print the market impact the walk measured.
//
//   npm install
//   node examples/node/run.js
//
// Every language example runs the same thin_book request and prints the same
// summary — that is the cross-language guarantee.
"use strict";

const { Impact } = require("wickra-impact");

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

// The thin_book worked example: the second bar's ask ladder is thin, so a market
// order walks up it and pays slippage a naive backtest would never see.
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

const impact = new Impact(SPEC);
const report = JSON.parse(impact.command(JSON.stringify({ cmd: "run", data: DATA })));
console.log(`wickra-impact ${impact.version()}`);
console.log(`avg slippage: ${report.impact_stats.avg_slippage_bps} bps`);
console.log(`entry price: ${report.report.trades[0].entry_price}`);
