"""A runnable Python example: back-test a buy-and-hold strategy against a thin
order book and print the market impact the walk measured.

    pip install wickra-impact
    python examples/python/run.py

Every language example runs the same thin_book request and prints the same
summary — that is the cross-language guarantee.
"""

import json

from wickra_impact import Impact

SPEC = json.dumps(
    {
        "strategy": {
            "spec_version": 1,
            "symbol": "IMPACT",
            "timeframe": "1h",
            "indicators": {},
            "entry": {"ge": [{"price": "close"}, 0]},
            "exit": {"in_position": True},
            "sizing": {"type": "fixed_qty", "qty": 10.0},
            "execution": {"order_type": "market", "fill_timing": "next_open"},
        },
        "book_model": {"kind": "orderbook_walk"},
        "participation_cap": 1.0,
        "latency_ms": 0,
    }
)

# The thin_book worked example: the second bar's ask ladder is thin, so a market
# order walks up it and pays slippage a naive backtest would never see.
DATA = {
    "candles": [
        {"time": 0, "open": 100, "high": 100, "low": 100, "close": 100, "volume": 1000},
        {"time": 3600, "open": 100, "high": 103, "low": 100, "close": 102, "volume": 1000},
    ],
    "books": [
        {"bids": [{"price": 99.9, "size": 100}], "asks": [{"price": 100.1, "size": 100}]},
        {
            "bids": [{"price": 99.9, "size": 100}],
            "asks": [
                {"price": 100.1, "size": 3},
                {"price": 100.3, "size": 3},
                {"price": 100.8, "size": 4},
            ],
        },
    ],
}


def main() -> None:
    impact = Impact(SPEC)
    report = json.loads(impact.command(json.dumps({"cmd": "run", "data": DATA})))
    print(f"wickra-impact {Impact.version()}")
    print(f"avg slippage: {report['impact_stats']['avg_slippage_bps']} bps")
    print(f"entry price: {report['report']['trades'][0]['entry_price']}")


if __name__ == "__main__":
    main()
