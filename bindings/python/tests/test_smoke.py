"""Smoke test: construct an impact backtest, run it, parse the report."""

import json

from wickra_impact import Impact, __version__

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


def _run(impact: Impact) -> dict:
    return json.loads(impact.command(json.dumps({"cmd": "run", "data": DATA})))


def test_run_returns_report_with_impact_stats() -> None:
    report = _run(Impact(json.dumps(SPEC)))
    # The walk sees the 44 bps of slippage a naive backtest hides.
    assert report["impact_stats"]["avg_slippage_bps"] == 44.0
    assert report["report"]["trades"][0]["entry_price"] == 100.44


def test_deterministic_across_instances() -> None:
    a = _run(Impact(json.dumps(SPEC)))
    b = _run(Impact(json.dumps(SPEC)))
    assert a == b


def test_set_spec_then_run() -> None:
    impact = Impact("{}")
    ok = json.loads(impact.command(json.dumps({"cmd": "set_spec", "spec": SPEC})))
    assert ok["ok"] is True
    report = _run(impact)
    assert "impact_stats" in report


def test_version() -> None:
    assert Impact.version() == __version__
