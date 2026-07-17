"""Determinism: the same request yields the byte-identical report string.

The full cross-language golden (asserting the response equals a blessed
golden/expected file) lands with the golden corpus in P-IMP-4; here we pin the
core invariant that a run is byte-reproducible from its inputs, which every
binding must preserve by forwarding the command string verbatim.
"""

import json

from wickra_impact import Impact

SPEC = {
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


def test_same_request_same_report_string() -> None:
    cmd = json.dumps({"cmd": "run", "data": DATA})
    a = Impact(json.dumps(SPEC)).command(cmd)
    b = Impact(json.dumps(SPEC)).command(cmd)
    assert a == b


def test_report_carries_the_measured_impact() -> None:
    cmd = json.dumps({"cmd": "run", "data": DATA})
    report = json.loads(Impact(json.dumps(SPEC)).command(cmd))
    assert report["impact_stats"]["avg_slippage_bps"] == 44.0
