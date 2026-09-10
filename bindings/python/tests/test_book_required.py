"""The book the walk needs, through the JSON command boundary.

There is no streaming half in IMPACT: a run is a batch computation. What stands
in its place is the property the whole repository exists for -- that slippage is
measured rather than guessed -- and the one way it can fail quietly.

`orderbook_walk` needs a book. Without one the core refuses the run. A binding
that swallowed that refusal would hand back a report showing zero slippage from
the model whose entire purpose is to find some, and nothing downstream could
tell the difference between "no impact" and "no book".
"""

import json

from wickra_impact import Impact

STRATEGY = {
    "spec_version": 1,
    "symbol": "IMPACT",
    "timeframe": "1h",
    "indicators": {},
    "entry": {"ge": [{"price": "close"}, 0]},
    "exit": {"in_position": True},
    "sizing": {"type": "fixed_qty", "qty": 10.0},
    "execution": {"order_type": "market", "fill_timing": "next_open"},
}

CANDLES = [
    {"time": 0, "open": 100, "high": 100, "low": 100, "close": 100, "volume": 1000},
    {"time": 3600, "open": 100, "high": 103, "low": 100, "close": 102, "volume": 1000},
]

BOOKS = [
    {"bids": [{"price": 99.9, "size": 100}], "asks": [{"price": 100.1, "size": 100}]},
    {
        "bids": [{"price": 99.9, "size": 100}],
        "asks": [
            {"price": 100.1, "size": 3},
            {"price": 100.3, "size": 3},
            {"price": 100.8, "size": 4},
        ],
    },
]


def _spec(model: dict) -> str:
    return json.dumps(
        {
            "strategy": STRATEGY,
            "book_model": model,
            "participation_cap": 1.0,
            "latency_ms": 0,
        }
    )


def _run(model: dict, data: dict) -> str:
    return Impact(_spec(model)).command(json.dumps({"cmd": "run", "data": data}))


def test_the_walk_refuses_a_run_with_no_book() -> None:
    response = _run({"kind": "orderbook_walk"}, {"candles": CANDLES})
    assert "book" in response, response
    assert '"impact_stats"' not in response, (
        "a walk with no book must be refused, not answered with zero slippage"
    )


def test_the_walk_measures_slippage_when_the_book_is_there() -> None:
    report = json.loads(
        _run({"kind": "orderbook_walk"}, {"candles": CANDLES, "books": BOOKS})
    )
    # Walking three ask levels for ten units pays 100.44, not the 100.10 top of
    # book a naive fill would take.
    assert report["impact_stats"]["avg_slippage_bps"] == 44.0
    assert report["report"]["trades"][0]["entry_price"] == 100.44


def test_an_analytic_model_needs_no_book() -> None:
    """`linear_impact` prices the order from a curve, so a bookless run is fine."""
    report = json.loads(
        _run({"kind": "linear_impact", "coef": 0.1}, {"candles": CANDLES})
    )
    assert "impact_stats" in report


def test_the_batch_run_is_reproducible() -> None:
    data = {"candles": CANDLES, "books": BOOKS}
    assert _run({"kind": "orderbook_walk"}, data) == _run({"kind": "orderbook_walk"}, data)
